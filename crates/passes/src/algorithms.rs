pub(crate) mod loop_detector;
mod split;
mod inline_extern_func;

pub use split::split_graph;
pub use inline_extern_func::inline_extern_func;
