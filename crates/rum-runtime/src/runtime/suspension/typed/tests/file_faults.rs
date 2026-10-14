use super::fault_support::*;
use super::*;
use crate::runtime::{
    FileIo, FileIoError, FileIoProvider, FileProbe, FileWriteMode, SessionRequestError,
};
use std::path::Path;

struct Files;
impl crate::runtime::host_payload::BoundedDestruction for Files {}
impl FileIo for Files {
    fn read_to_string(&self, path: &Path) -> Result<String, FileIoError> {
        match path.file_name().and_then(|name| name.to_str()) {
            Some("Outer.rum") => Ok("Global`fileMarker=10\nGet[\"Inner`\"]\nGlobal`fileLater=1".into()),
            Some("Inner.rum") => Ok("Global`fileMarker=20\nWithCleanup[Block[{fileScope=inside},SlotTrip[];Global`fileLater=2],Print[\"file cleanup\"]]".into()),
            _ => Err(FileIoError::new("missing memory file")),
        }
    }
    fn probe(&self, path: &Path) -> FileProbe {
        if self.read_to_string(path).is_ok() {
            FileProbe::file(path.to_owned())
        } else {
            FileProbe::missing()
        }
    }
    fn write_string(&self, _: &Path, _: &str, _: FileWriteMode) -> Result<(), FileIoError> {
        Err(FileIoError::new("read-only memory file"))
    }
}

#[test]
fn slot_fault_rolls_back_nested_memory_files_and_public_batch_owner() {
    for kind in [
        FaultKind::ValueForState,
        FaultKind::StateForValue,
        FaultKind::WrongState,
    ] {
        for public_batch in [false, true] {
            let (mut s, events) = session(kind, false);
            s.set_file_io(Some(FileIoProvider::new(Files)));
            s.eval_request("fileMarker=0;fileLater=0;fileScope=outside")
                .unwrap();
            let probe = s.continuation_fault_probe();
            if public_batch {
                assert!(matches!(
                    s.eval_file("Outer.rum"),
                    Err(EvalError::InputRequired { .. })
                ));
            } else {
                assert!(matches!(
                    s.eval_request("Get[\"Outer`\"]").unwrap_err().error,
                    SessionRequestError::InputRequired { .. }
                ));
            }
            assert_eq!(s.load_stack.len(), 2);
            super::accounting::reconcile(&s);
            if public_batch {
                let result = s.resume_suspended_evaluation("answer");
                let error = s.continuation_fault_outcome(result).unwrap_err();
                kind.assert_eval(&error);
            } else {
                kind.assert_request(
                    &s.resume_eval_request_with_input("answer")
                        .unwrap_err()
                        .error,
                );
            }
            assert_eq!(probe.occurrences(), 1);
            assert_eq!(
                *events.lock().unwrap(),
                ["emit", "consumer", "suffix-one", "suffix-two"]
            );
            assert_eq!(s.prints, ["file cleanup"]);
            restored(&mut s);
            assert_eq!(
                s.eval_request("{fileMarker,fileLater,fileScope}")
                    .unwrap()
                    .value
                    .to_string(),
                "List[0, 0, outside]"
            );
            assert_eq!(
                probe.occurrences(),
                1,
                "rollback cannot erase the first fault"
            );
        }
    }
}
