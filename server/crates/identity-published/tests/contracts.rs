use ordering_food_identity_published::{SubjectRef, SubjectStatus};

#[test]
fn subject_ref_tracks_subject_identity_and_status() {
    let subject = SubjectRef::new("subject-1", SubjectStatus::Active);

    assert_eq!(subject.subject_id(), "subject-1");
    assert_eq!(subject.status(), SubjectStatus::Active);
}
