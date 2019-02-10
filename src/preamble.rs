    fn get_preamble_at(&self, lines: &[Line], start_index: usize) -> Option<P>;
    fn get_preamble_at(&self, lines: &[Line], start_index: usize) -> Option<GitPreamble> {
    fn get_preamble_at(&self, lines: &[Line], start_index: usize) -> Option<DiffPreamble> {
    diff_preamble_parser: DiffPreambleParser,
            diff_preamble_parser: DiffPreambleParser::new(),
    pub fn get_preamble_at(&self, lines: &[Line], start_index: usize) -> Option<Preamble> {
        } else if let Some(preamble) = self
            .diff_preamble_parser
            .get_preamble_at(lines, start_index)
        {
            Some(Preamble::Diff(preamble))