pub mod ingestion {
    pub mod file_scanner;
}
pub mod analysis {
    pub mod dependency_extractor;
    pub mod dependency_graph;
    pub mod cycle_detector;
}
pub mod report {
    pub mod text_reporter;
}
