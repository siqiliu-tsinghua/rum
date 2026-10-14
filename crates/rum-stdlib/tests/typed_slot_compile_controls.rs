//! Paired compiler controls for S1 sealing and the S2 live suspension routes.
//! Diagnostics are parsed as Cargo JSON, including exact primary source spans.
use std::{path::Path, process::Command};

#[test]
fn typed_slot_compile_controls() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let feature = if cfg!(feature = "hash") {
        "default"
    } else {
        "no-default"
    };
    let output = Command::new("python3")
        .current_dir(root)
        .args([
            "scripts/run-bounded.py",
            "--timeout",
            "880",
            "--",
            "python3",
            "-c",
            HARNESS,
        ])
        .arg(root)
        .arg(feature)
        .output()
        .expect("bounded compiler controls");
    print!("{}", String::from_utf8_lossy(&output.stdout));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

const HARNESS: &str = r###"
from pathlib import Path
import json, os, shutil, subprocess, sys

root=Path(sys.argv[1]); feature=sys.argv[2]
scratch=root/'.temp/brief-608/compile-controls'; scratch.mkdir(parents=True,exist_ok=True)
logs=scratch/feature; logs.mkdir(exist_ok=True)
external=scratch/'external'; (external/'src').mkdir(parents=True,exist_ok=True)
copy=scratch/'private-workspace'; copy.mkdir(exist_ok=True)
for name in ['Cargo.toml','Cargo.lock','rust-toolchain.toml']:
    if (root/name).exists(): shutil.copy2(root/name,copy/name)
for name in ['crates','docs']:
    shutil.copytree(root/name,copy/name,dirs_exist_ok=True)
for name in ['README.md','ARCHITECTURE.md','DISCLAIMER.md','TODO.md']:
    shutil.copy2(root/name,copy/name)
(external/'Cargo.toml').write_text('''[package]
name = "typed-slot-controls"
version = "0.0.0"
edition = "2024"
[workspace]
[dependencies]
rum-runtime = { path = '''+json.dumps(str(root/'crates/rum-runtime'))+''' }
rum-core = { path = '''+json.dumps(str(root/'crates/rum-core'))+''' }
''')

support='''
use std::{marker::PhantomData, sync::Arc};
use rum_core::Expr;
use rum_runtime::kernel_api::*;
use rum_runtime::RuntimeInputRequest;
struct Frame<I:SlotValue,O:SlotValue>(PhantomData<fn(I)->O>);
fn frame<I:SlotValue,O:SlotValue>()->Frame<I,O>{Frame(PhantomData)}
impl<I:SlotValue,O:SlotValue> FrameRootTrace for Frame<I,O>{
 fn trace_roots(&self,_:&mut dyn FrameRootVisitor){}
}
impl<I:SlotValue,O:SlotValue> FrameAbandon for Frame<I,O>{
 fn abandon(self:Arc<Self>,_:&mut AbandonmentContext<'_>)->FrameAbandonAction{FrameAbandonAction::Discard}
}
impl<I:SlotValue,O:SlotValue> TypedFrame for Frame<I,O>{
 type Input=I; type Output=O;
 fn resume<'r>(self:Arc<Self>,_:&mut Session,_:Result<I,EvalError>,_:ContinueToken<'r,Self>)->TypedAction<O>{TypedAction::complete(Err(EvalError::Abort))}
}
struct Finish<O:SlotValue>(PhantomData<fn()->O>);
impl<O:SlotValue> SlotFinalizer for Finish<O>{
 type Output=O;
 fn finish(self:Box<Self>)->Result<O,EvalError>{Err(EvalError::Abort)}
}
'''
cases=[]; modules=[]
ext_file=external/'src/lib.rs'
def external_case(id,body,old,new,code,needle,category=''):
    module='mod '+id.replace('-','_')+' { use super::*;\n'+body+'\n}\n'
    assert module.count(old)==1,(id,old)
    modules.append(module)
    cases.append(dict(id=id,kind='external',file=ext_file,old=old,new=new,code=code,needle=needle,category=category,marker='@'+id))

external_case('first-chain-input', '''type Input=Expr; // first input
fn check(){let actual:Result<Expr,EvalError>=Ok(Expr::string("x"));
let _=slot_witness(&actual).chain(frame::<Input,Expr>()); // @first-chain-input
}''','type Input=Expr; // first input','type Input=Option<Expr>; // first input','E0271','frame::<Input,Expr>()')
external_case('chain-adjacency', '''type Input=Option<Expr>; // adjacent input
fn check(){let actual:Result<Expr,EvalError>=Ok(Expr::string("x"));
let _=slot_witness(&actual).chain(frame::<Expr,Option<Expr>>()).then(frame::<Input,Expr>()); // @chain-adjacency
}''','type Input=Option<Expr>; // adjacent input','type Input=bool; // adjacent input','E0271','frame::<Input,Expr>()')
external_case('unbound-chain-constructor', '''fn check(){let actual:Result<Expr,EvalError>=Ok(Expr::string("x"));
let _:FrameChain<Expr,Expr>=slot_witness(&actual).chain(frame::<Expr,Expr>()); // @unbound-chain-constructor
}''','slot_witness(&actual).chain(frame::<Expr,Expr>()); // @unbound','FrameChain::new(frame::<Expr,Expr>()); // @unbound','E0599','new')
external_case('evaluate-input', '''type Input=Expr; // evaluate input
fn check(){let _=TypedAction::<Expr>::evaluate(Expr::string("x"),frame::<Input,Expr>()); // @evaluate-input
}''','type Input=Expr; // evaluate input','type Input=Option<Expr>; // evaluate input','E0271','frame::<Input,Expr>()')
external_case('evaluate-output', '''type Output=Expr; // evaluate output
fn check(){let _=TypedAction::<Expr>::evaluate(Expr::string("x"),frame::<Expr,Output>()); // @evaluate-output
}''','type Output=Expr; // evaluate output','type Output=Option<Expr>; // evaluate output','E0271','frame::<Expr,Output>()')
external_case('finalizer-output', '''type Output=Expr; // finalizer output
fn check(){let _=TypedAction::<Expr>::finalize(Finish::<Output>(PhantomData)); // @finalizer-output
}''','type Output=Expr; // finalizer output','type Output=Option<Expr>; // finalizer output','E0271','Finish::<Output>(PhantomData)')
external_case('opaque-action', '''fn check(){let _=TypedAction::<Expr>::complete(Ok(Expr::string("x"))); // @opaque-action
}''','::complete(Ok(Expr::string("x"))); // @opaque-action','::Complete(Ok(Expr::string("x"))); // @opaque-action','E0599','Complete')
external_case('token-no-clone', '''fn check<'r>(token:ContinueToken<'r,Frame<Expr,Expr>>){
let _moved=token; // @token-no-clone
}''','let _moved=token;','let _moved=token.clone();','E0599','clone')
external_case('token-no-mint', '''fn check<'r>(token:ContinueToken<'r,Frame<Expr,Expr>>,f:Frame<Expr,Expr>){
let _moved=token; // @token-no-mint
}''','let _moved=token; // @token-no-mint','let _moved=ContinueToken::of(f); // @token-no-mint','E0599','of')
external_case('token-no-escape', '''fn check<'r>(token:ContinueToken<'r,Frame<Expr,Expr>>)->ContinueToken<'r,Frame<Expr,Expr>>{
token // @token-no-escape
}''',"->ContinueToken<'r,Frame<Expr,Expr>>","->ContinueToken<'static,Frame<Expr,Expr>>",None,'token','lifetime may not live long enough')
external_case('sealed-extension', '''struct LocalPayload;
// @sealed-extension
fn check(){let _=LocalPayload;}''','// @sealed-extension','impl SlotValue for LocalPayload {} // @sealed-extension','E0277','LocalPayload','Codec')
external_case('codec-key-private', '''fn check<T:SlotValue>(value:T){let _=TypedAction::complete(Ok(value));
// @codec-key-private
}''','// @codec-key-private','use rum_runtime::runtime::suspension::typed::channel::CodecKey; // @codec-key-private','E0603','runtime::suspension')
external_case('opaque-selector-pass', '''type Input=EvaluatedPartSelectors; // opaque input
fn check(actual:Result<EvaluatedPartSelectors,EvalError>){
let _=slot_witness(&actual).chain(frame::<Input,Expr>()); // @opaque-selector-pass
}''','type Input=EvaluatedPartSelectors; // opaque input','type Input=Option<Expr>; // opaque input','E0271','frame::<Input,Expr>()')
external_case('mode-not-frame-chosen', '''fn check(actual:Result<Option<Expr>,EvalError>){
let _=slot_witness(&actual).chain(frame::<Option<Expr>,Expr>()); // @mode-not-frame-chosen
}''','chain(frame::<Option<Expr>,Expr>()); // @mode-not-frame-chosen','chain(frame::<Option<Expr>,Expr>()).with_mode::<()>(); // @mode-not-frame-chosen','E0599','with_mode')
external_case('mode-private-selection', '''fn check<T:SlotValue>(value:T){let _=TypedAction::complete(Ok(value));
// @mode-private-selection
}''','// @mode-private-selection','use rum_runtime::runtime::suspension::typed::channel::LegacyMode; // @mode-private-selection','E0603','runtime::suspension')
external_case('slot-mismatch-private', '''fn inspect(error:EvalError)->Option<(&'static str,&'static str)>{
 match error { EvalError::ContinuationSlotMismatch(fault)=>{
  let _: &rum_runtime::SlotMismatch=&fault;
  let _: &SlotMismatch=&fault;
  Some((fault.expected(),fault.got()))
 }, _=>None }
}
fn check(error:EvalError){let _=inspect(error); // @slot-mismatch-private
}''','let _=inspect(error);','let _=EvalError::ContinuationSlotMismatch(rum_runtime::SlotMismatch{expected:"expected",got:"got"});','E0451','expected:',category='private')
external_case('input', '''type Input=Expr; // suspend input
fn check(s:&mut Session,actual:Result<Expr,EvalError>,request:RuntimeInputRequest)->Result<Expr,EvalError>{
slot_witness(&actual).suspend(s,request,frame::<Input,Expr>()) // @input
}''','type Input=Expr; // suspend input','type Input=Option<Expr>; // suspend input','E0271','frame::<Input,Expr>()')
external_case('output-return', '''type Output=Option<Expr>; // return output
fn check(s:&mut Session,actual:Result<Expr,EvalError>,request:RuntimeInputRequest)->Result<Output,EvalError>{
slot_witness(&actual).suspend(s,request,frame::<Expr,Option<Expr>>()) // @output-return
}''','type Output=Option<Expr>; // return output','type Output=Expr; // return output','E0308','slot_witness(&actual).suspend',category='mismatched types')
external_case('chain-last-output', '''type Last=Expr; // chain final output
fn check(s:&mut Session,actual:Result<Expr,EvalError>,request:RuntimeInputRequest)->Result<Expr,EvalError>{
slot_witness(&actual).chain(frame::<Expr,Option<Expr>>()).then(frame::<Option<Expr>,Last>()).suspend(s,request) // @chain-last-output
}''','type Last=Expr; // chain final output','type Last=bool; // chain final output','E0308','slot_witness(&actual).chain',category='mismatched types')
external_case('token-fresh-type', '''struct A { cursor:usize }
struct B { cursor:usize }
impl FrameRootTrace for A {fn trace_roots(&self,_:&mut dyn FrameRootVisitor){}}
impl FrameRootTrace for B {fn trace_roots(&self,_:&mut dyn FrameRootVisitor){}}
impl FrameAbandon for A {fn abandon(self:Arc<Self>,_:&mut AbandonmentContext<'_>)->FrameAbandonAction{FrameAbandonAction::Discard}}
impl FrameAbandon for B {fn abandon(self:Arc<Self>,_:&mut AbandonmentContext<'_>)->FrameAbandonAction{FrameAbandonAction::Discard}}
impl TypedFrame for A {type Input=Expr;type Output=Expr;fn resume<'r>(self:Arc<Self>,_:&mut Session,value:Result<Expr,EvalError>,_:ContinueToken<'r,Self>)->TypedAction<Expr>{TypedAction::complete(value)}}
impl TypedFrame for B {type Input=Expr;type Output=Expr;fn resume<'r>(self:Arc<Self>,_:&mut Session,value:Result<Expr,EvalError>,_:ContinueToken<'r,Self>)->TypedAction<Expr>{TypedAction::complete(value)}}
fn check<'r>(s:&mut Session,request:RuntimeInputRequest,token:ContinueToken<'r,A>,cursor:usize)->ResumeStop<Expr>{
token.suspend(s,request,A{cursor:cursor+1}) // @token-fresh-type
}''','token.suspend(s,request,A{cursor:cursor+1})','token.suspend(s,request,B{cursor:cursor+1})','E0308','B{cursor:cursor+1}',category='mismatched types')
external_case('deferred-final-output', '''type Output=Option<Expr>; // deferred owner output
fn check<'r>(s:&mut Session,request:RuntimeInputRequest,token:ContinueToken<'r,Frame<Expr,Option<Expr>>>)->Result<usize,ResumeStop<Output>>{
Err(token.suspend(s,request,frame::<Expr,Option<Expr>>())) // @deferred-final-output
}''','type Output=Option<Expr>; // deferred owner output','type Output=Expr; // deferred owner output','E0308','token.suspend(s,request,frame::<Expr,Option<Expr>>())',category='mismatched types')
external_case('generic-frame', '''fn check<F:TypedFrame<Input=Expr,Output=Expr>>(s:&mut Session,actual:Result<Expr,EvalError>,request:RuntimeInputRequest,frame:F)->Result<Expr,EvalError>{
slot_witness(&actual).suspend(s,request,frame) // @generic-frame
}''','F:TypedFrame<Input=Expr,Output=Expr>','F:TypedFrame<Input=Expr>','E0308','slot_witness(&actual).suspend',category='mismatched types')
external_case('closure-return', '''fn check(s:&mut Session,actual:Result<Expr,EvalError>,request:RuntimeInputRequest){
let _=(||->Result<Option<Expr>,EvalError>{
slot_witness(&actual).suspend(s,request,frame::<Expr,Option<Expr>>()) // @closure-return
})();
}''','||->Result<Option<Expr>,EvalError>','||->Result<Expr,EvalError>','E0308','slot_witness(&actual).suspend',category='mismatched types')
ext_file.write_text('#![allow(dead_code,unused_imports,unused_variables)]\n'+support+'\n'.join(modules))

runtime=copy/'crates/rum-runtime/src/runtime'
typed=runtime/'suspension/typed.rs'; registry=runtime/'suspension/typed/registry.rs'; channel=runtime/'suspension/typed/channel.rs'
registry.write_text('use super::channel::StateMode;\n'+registry.read_text())
typed.write_text(typed.read_text()+'''
#[cfg(test)]
mod mode_controls {
 use super::channel::*;
 use rum_core::Expr;
 struct NewCarrier;
 impl Codec for NewCarrier { type Mode=StateMode; } // @mode-new-state-only
 impl StateCodec for NewCarrier {
  fn pack(self,_:CodecKey)->ContinuationState { ContinuationState::Bool(false) }
  fn unpack(_:ContinuationState,_:CodecKey)->Result<Self,ContinuationState> { Ok(Self) }
 }
 struct NarrowLegacy;
 impl LegacyCodec for NarrowLegacy {
  fn old_encode(self,_:CodecKey)->Expr { Expr::string("fixture") }
  fn old_decode(_:Expr,_:CodecKey)->Result<Self,Expr> { Ok(Self) } // @mode-legacy-provider-narrow
 }
 struct NarrowState;
 impl StateCodec for NarrowState {
  fn pack(self,_:CodecKey)->ContinuationState { ContinuationState::Bool(false) }
  fn unpack(_:ContinuationState,_:CodecKey)->Result<Self,ContinuationState> { Ok(Self) } // @mode-state-provider-narrow
 }
}
''')
sibling=runtime/'mod.rs'
sibling.write_text(sibling.read_text()+'\n#[cfg(test)]\nmod sibling_control {\n'+support.replace('rum_runtime::','crate::runtime::')+'''
fn check(){ let _=TypedAction::<Expr>::evaluate(Expr::string("x"),frame::<Expr,Expr>()); } // @sibling-erased-private
}
''')
def private_case(id,file,old,new,code,needle,marker=None,category=''):
    assert file.read_text().count(old)==1,(id,old)
    cases.append(dict(id=id,kind='private',file=file,old=old,new=new,code=code,needle=needle,category=category,marker=marker or '@'+id))
private_case('sibling-erased-private',sibling,'TypedAction::<Expr>::evaluate(Expr::string("x"),frame::<Expr,Expr>())','crate::runtime::suspension::typed::adapter::erase(frame::<Expr,Expr>())','E0603','typed::adapter')
private_case('mode-expr-fixed',registry,'impl Codec for Expr {\n    type Mode = ValueMode;','impl Codec for Expr {\n    type Mode = StateMode; // @mode-expr-fixed','E0277','StateMode',category='StateCodec')
private_case('mode-new-state-only',typed,'type Mode=StateMode; } // @mode-new-state-only','type Mode=LegacyMode; } // @mode-new-state-only','E0277','LegacyMode',category='LegacyCodec')
private_case('mode-one-associated-type',registry,'impl Codec for Option<Expr> {\n    type Mode = LegacyMode;','impl Codec for Option<Expr> {\n    type Mode = LegacyMode;\n    type Mode = LegacyMode; // @mode-one-associated-type','E0201','type Mode = LegacyMode;')
private_case('mode-no-codec-override',registry,'impl Codec for Option<Expr> {\n    type Mode = LegacyMode;','impl Codec for Option<Expr> {\n    type Mode = LegacyMode;\n    fn decode(slot: super::channel::Slot, _: CodecKey) -> Result<Self, super::channel::Slot> { Err(slot) } // @mode-no-codec-override','E0407','fn decode')
private_case('mode-legacy-provider-narrow',typed,'fn old_decode(_:Expr,_:CodecKey)','fn old_decode(_:Slot,_:CodecKey)','E0053','Slot',category='incompatible type for trait')
private_case('mode-state-provider-narrow',typed,'fn unpack(_:ContinuationState,_:CodecKey)->Result<Self,ContinuationState> { Ok(Self) } // @mode-state-provider-narrow','fn unpack(_:Slot,_:CodecKey)->Result<Self,ContinuationState> { Ok(Self) } // @mode-state-provider-narrow','E0053','Slot',category='incompatible type for trait')
assert len(cases)==30
base={case['file']:case['file'].read_text() for case in cases}

def compile(kind,label):
    work=external if kind=='external' else copy
    args=['cargo','check','--message-format=json'] if kind=='external' else ['cargo','test','-p','rum-runtime','--lib','--no-run','--message-format=json']
    if feature=='no-default': args+=['--no-default-features']
    env=os.environ.copy(); env['CARGO_TARGET_DIR']=str(scratch/('target-'+kind)); env['CARGO_NET_OFFLINE']='true'; env['CARGO_BUILD_JOBS']='1'; env['RUST_TEST_THREADS']='1'
    command=['python3',str(root/'scripts/run-bounded.py'),'--timeout','750','--',*args]
    (logs/(label+'.command.json')).write_text(json.dumps(dict(cwd=str(work),command=command,feature=feature),indent=2)+'\n')
    result=subprocess.run(command,cwd=work,env=env,capture_output=True,text=True)
    (logs/(label+'.jsonl')).write_text(result.stdout)
    (logs/(label+'.stderr')).write_text(result.stderr)
    diagnostics=[]
    for line in result.stdout.splitlines():
        try: record=json.loads(line)
        except ValueError: continue
        if record.get('reason')=='compiler-message' and record['message']['level']=='error': diagnostics.append(record['message'])
    return result.returncode,diagnostics

# Each positive fixture is present and compiled before any mutation in its group.
# Shared positive builds avoid repeatedly rebuilding the same runtime test crate.
report=[]
for kind in ['external','private']:
    status,errors=compile(kind,kind+'-positive')
    assert status==0 and not errors,(kind,'positive failed',errors)
    for case in [c for c in cases if c['kind']==kind]:
        file=case['file']; changed=base[file].replace(case['old'],case['new'],1)
        file.write_text(changed)
        (logs/(case['id']+'.positive.rs')).write_text(base[file])
        (logs/(case['id']+'.negative.rs')).write_text(changed)
        try:
            status,errors=compile(kind,case['id'])
            assert status!=0 and errors,(case['id'],'negative compiled or instrumentation failed')
            lines=changed.splitlines(); marked=[(i+1,line) for i,line in enumerate(lines) if case['marker'] in line]
            assert len(marked)==1,(case['id'],marked)
            line,text=marked[0]; column=text.index(case['needle'])+1
            expected_file=file.relative_to(external if kind=='external' else copy).as_posix()
            matches=[]
            for error in errors:
                code=(error.get('code') or {}).get('code')
                if code!=case['code'] or case['category'] not in error['message']: continue
                for span in error['spans']:
                    if span['is_primary'] and span['file_name'].replace('\\','/').endswith(expected_file) and span['line_start']==line and span['column_start']==column:
                        matches.append((error,span))
            assert matches,(case['id'],f'expected {case["code"]} at {expected_file}:{line}:{column}',errors)
            def related(error):
                code=(error.get('code') or {}).get('code')
                if code==case['code']: return True
                # The real registry unit equality witness also rejects this
                # single mode mutation. Retain it and pin its independent span.
                if case['id']=='mode-expr-fixed' and code=='E0271' and '<Expr as Codec>::Mode == ValueMode' in error['message']:
                    return any(span['is_primary'] and span['file_name'].endswith('/suspension/typed/tests/registry.rs') and any('value::<Expr>();' in text['text'] for text in span['text']) for span in error['spans'])
                return False
            assert all(related(error) for error in errors),(case['id'],'unrelated diagnostic',errors)
            if case['id']=='slot-mismatch-private':
                # Pin both private fields' complete primary spans, not just
                # the start of the first diagnostic at the constructor.
                expected_spans=sorted((line,text.index(field)+1,line,text.index(field)+1+len(field)) for field in ['expected:"expected"','got:"got"'])
                actual_spans=sorted((span['line_start'],span['column_start'],span['line_end'],span['column_end']) for error in errors for span in error['spans'] if span['is_primary'] and span['file_name']==expected_file)
                assert len(errors)==2 and actual_spans==expected_spans,(case['id'],expected_spans,errors)
            error,span=matches[0]
            receipt=dict(id=case['id'],positive=kind+'-positive.jsonl',negative=case['id']+'.jsonl',code=case['code'],category=error['message'],primary_span=span)
            if case['id']=='slot-mismatch-private': receipt['privacy_spans']=actual_spans
            report.append(receipt)
            print(f'PAIR {case["id"]}: positive PASS; negative {case["code"] or "lifetime"} at {expected_file}:{line}:{column}',flush=True)
        finally: file.write_text(base[file])
(logs/'results.json').write_text(json.dumps(report,indent=2)+'\n')
assert len(report)==30
print('COMPILE CONTROLS: 30 positive / 30 intended rejections (22 S1 + 7 S2 + opaque fault); raw Cargo JSON and primary spans retained',flush=True)
"###;
