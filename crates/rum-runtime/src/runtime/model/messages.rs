use super::*;
use crate::runtime::kernel_api::{FrameRootTrace, FrameRootVisitor};

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub enum DefinitionKind {
    OwnValue,
    DelayedOwnValue,
    UpValue,
    DownValue,
    SubValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub enum RuleApplicationSource {
    UpValue,
    DownValue,
    SubValue,
    Replace,
    ReplaceList,
    ReplaceAt,
    ReplaceAll,
    ReplaceAllUnheld,
    ReplaceRepeated,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub enum TraceEvent {
    EvaluationPass {
        before: Expr,
        after: Expr,
    },
    DefinitionStored {
        symbol: String,
        kind: DefinitionKind,
        value: Expr,
    },
    RuleApplied {
        source: RuleApplicationSource,
        input: Expr,
        rule: Expr,
        result: Expr,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub struct RuntimeMessage {
    pub symbol: String,
    pub tag: String,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FrameRootTrace)]
pub enum RuntimeInputKind {
    InputString,
    Input,
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub struct RuntimeInputRequest {
    pub kind: RuntimeInputKind,
    pub prompt: String,
    pub password: bool,
}

impl RuntimeInputRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            kind: RuntimeInputKind::InputString,
            prompt: prompt.into(),
            password: false,
        }
    }

    pub fn input(mut self) -> Self {
        self.kind = RuntimeInputKind::Input;
        self
    }

    pub fn input_string(mut self) -> Self {
        self.kind = RuntimeInputKind::InputString;
        self
    }

    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct MessageSelector {
    pub(in crate::runtime) symbol: String,
    pub(in crate::runtime) tag: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, FrameRootTrace)]
pub(in crate::runtime) struct MessageNameKey {
    pub(in crate::runtime) symbol: Symbol,
    pub(in crate::runtime) tag: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, FrameRootTrace)]
pub(in crate::runtime) struct MessageDefinition {
    pub(in crate::runtime) immediate: Option<Expr>,
    pub(in crate::runtime) delayed: Option<Expr>,
}

impl fmt::Display for RuntimeMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}: {}", self.symbol, self.tag, self.text)
    }
}

impl fmt::Display for RuntimeInputRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.prompt.is_empty() {
            f.write_str("input required")
        } else {
            write!(f, "input required: {}", self.prompt)
        }
    }
}

impl MessageSelector {
    pub(in crate::runtime) fn matches(&self, message: &RuntimeMessage) -> bool {
        self.symbol == message.symbol
            && self
                .tag
                .as_ref()
                .map_or(true, |tag| message.tag.as_str() == tag.as_str())
    }

    pub(in crate::runtime) fn covers(&self, other: &MessageSelector) -> bool {
        self.symbol == other.symbol
            && self.tag.as_ref().map_or(true, |tag| {
                other.tag.as_ref().is_some_and(|other| other == tag)
            })
    }
}
#[cfg(test)]
const _: fn() = || {
    fn assert_traces<T: FrameRootTrace>() {}
    assert_traces::<MessageDefinition>();
    assert_traces::<MessageNameKey>();
    assert_traces::<RuntimeInputKind>();
    assert_traces::<RuntimeInputRequest>();
    assert_traces::<RuntimeMessage>();
};
