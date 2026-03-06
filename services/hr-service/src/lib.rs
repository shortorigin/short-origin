use contracts::ServiceBoundaryV1;

#[must_use]
pub fn service_boundary() -> ServiceBoundaryV1 {
    ServiceBoundaryV1 {
        service_name: "hr-service".to_owned(),
        domain: "hr_talent".to_owned(),
        approved_workflows: vec!["hiring_approval".to_owned(), "payroll".to_owned()],
        owned_aggregates: vec![
            "personnel_record".to_owned(),
            "compensation_record".to_owned(),
        ],
    }
}
