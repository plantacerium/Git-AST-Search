use crate::modules::Filter;

pub fn execute_search_command(pattern: &str, filters: Vec<Filter>) -> String {
    let filter_str = if filters.is_empty() {
        String::new()
    } else {
        format!(
            " with filters: {}",
            filters
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    format!("Searching pattern: '{}{}'", pattern, filter_str)
}
