/// A set of filters and optional pagination to apply to a query.
/// All filters are AND-conjoined. OR conditions are not supported.
/// `F` is a context-local field enum (e.g. `UserField`, `TaskField`).
#[derive(Debug, Clone, PartialEq)]
pub struct Criteria<F> {
    pub filters: Vec<Filter<F>>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

/// A single filter triple: (field, operator, value).
/// `value` is always `String`; adapters parse it to the native type.
/// Invalid values must return AppError with code `E_INVALID_FILTER_VALUE`.
#[derive(Debug, Clone, PartialEq)]
pub struct Filter<F> {
    pub field: F,
    pub op: Op,
    pub value: String,
}

/// Comparison operators. Exact semantics are determined by the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Eq,
    NotEq,
    Like, // substring / pattern match — semantics determined by the adapter
    Gt,
    Lt,
    Gte,
    Lte,
}

impl<F> Criteria<F> {
    pub fn new() -> Self {
        Self {
            filters: vec![],
            limit: None,
            offset: None,
        }
    }

    pub fn filter(mut self, f: Filter<F>) -> Self {
        self.filters.push(f);
        self
    }

    pub fn limit(mut self, n: u64) -> Self {
        self.limit = Some(n);
        self
    }

    pub fn offset(mut self, n: u64) -> Self {
        self.offset = Some(n);
        self
    }
}

impl<F> Default for Criteria<F> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum TestField {
        Name,
        Age,
    }

    #[test]
    fn empty_criteria_has_no_filters() {
        let c: Criteria<TestField> = Criteria::new();
        assert!(c.filters.is_empty());
        assert!(c.limit.is_none());
        assert!(c.offset.is_none());
    }

    #[test]
    fn builder_adds_filters_and_pagination() {
        let c = Criteria::new()
            .filter(Filter {
                field: TestField::Name,
                op: Op::Eq,
                value: "alice".into(),
            })
            .limit(10)
            .offset(20);
        assert_eq!(c.filters.len(), 1);
        assert_eq!(c.limit, Some(10));
        assert_eq!(c.offset, Some(20));
    }

    #[test]
    fn multiple_filters_are_all_present() {
        let c = Criteria::new()
            .filter(Filter {
                field: TestField::Name,
                op: Op::Eq,
                value: "alice".into(),
            })
            .filter(Filter {
                field: TestField::Age,
                op: Op::Gte,
                value: "18".into(),
            });
        assert_eq!(c.filters.len(), 2);
    }
}
