use abstract_domain_derive::AbstractDomain;
use move_binary_format::file_format::CodeOffset;
use move_model::model::FunctionEnv;
use move_stackless_bytecode::{
    dataflow_analysis::{DataflowAnalysis, TransferFunctions},
    dataflow_domains::{AbstractDomain, JoinResult, Plus2},
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::Bytecode,
    stackless_control_flow_graph::StacklessControlFlowGraph,
};
use std::{collections::BTreeMap, fmt::Display};

/// true: definitely aborting later
/// false: definitely not aborting later
/// top: maybe abort later or not
/// bot: neither aborting nor returning later
#[derive(AbstractDomain, Clone)]
pub struct AbortState(Plus2<bool>);

impl AbortState {
    /// Set state from booleans
    fn set_bool(&mut self, b: bool) {
        self.0 = Plus2::Mid(b);
    }

    /// Set state to definitely abort
    fn set_abort(&mut self) {
        self.set_bool(true)
    }

    /// Set state to definitely not abort
    fn set_not_abort(&mut self) {
        self.set_bool(false)
    }

    /// Returns the bottom element
    fn bot() -> Self {
        Self(Plus2::Bot)
    }
}

impl Display for AbortState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self.0 {
            Plus2::Top => "maybe",
            Plus2::Mid(true) => "definitely abort",
            Plus2::Mid(false) => "definitely not abort",
            Plus2::Bot => "not aborting or returning",
        })
    }
}

/// Before and after abort state at a program point
#[derive(Clone)]
struct AbortStateAtCodeOffset {
    before: AbortState,
	#[allow(dead_code)]
    after: AbortState,
}

impl AbortStateAtCodeOffset {
    pub fn new(before: AbortState, after: AbortState) -> Self {
        Self { before, after }
    }
}

#[derive(Clone)]
struct AbortStateAnnotation(BTreeMap<CodeOffset, AbortStateAtCodeOffset>);

pub struct AbortAnalysis {}

impl AbortAnalysis {
    /// Returns the state per instruction of the given function
    fn analyze(&self, target: &FunctionTarget) -> BTreeMap<CodeOffset, AbortStateAtCodeOffset> {
        let code = target.get_bytecode();
        let cfg = StacklessControlFlowGraph::new_backward(code, true);
        let state_map = self.analyze_function(AbortState::bot(), code, &cfg);
        self.state_per_instruction(state_map, code, &cfg, |before, after| {
            AbortStateAtCodeOffset::new(before.clone(), after.clone())
        })
    }
}

impl TransferFunctions for AbortAnalysis {
    type State = AbortState;

    const BACKWARD: bool = true;

    fn execute(&self, state: &mut Self::State, instr: &Bytecode, _offset: CodeOffset) {
        match instr {
            Bytecode::Abort(..) => state.set_abort(),
            Bytecode::Ret(..) => state.set_not_abort(),
            _ => {},
        }
    }
}

impl DataflowAnalysis for AbortAnalysis {}

pub struct AbortAnalysisProcessor {}

impl FunctionTargetProcessor for AbortAnalysisProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        fun_env: &FunctionEnv,
        mut data: FunctionData,
        _scc_opt: Option<&[FunctionEnv]>,
    ) -> FunctionData {
        if fun_env.is_native() {
            return data;
        }
        let target = FunctionTarget::new(fun_env, &data);
        let analysis = AbortAnalysis {};
        let annotations = AbortStateAnnotation(analysis.analyze(&target));
        data.annotations.set(annotations, true);
        data
    }

    fn name(&self) -> String {
        "AbortAnalysisProcessor".to_owned()
    }
}

impl AbortAnalysisProcessor {
    pub fn register_formatters(target: &FunctionTarget) {
        target.register_annotation_formatter(Box::new(format_abort_state_annotation))
    }
}

/// Formats abort state annotations
pub fn format_abort_state_annotation(
    target: &FunctionTarget,
    code_offset: CodeOffset,
) -> Option<String> {
    let AbortStateAnnotation(state_per_instr) =
        target.get_annotations().get::<AbortStateAnnotation>()?;
    let AbortStateAtCodeOffset { before, .. } = state_per_instr.get(&code_offset)?;
    Some(format!("abort state: {}", before))
}
