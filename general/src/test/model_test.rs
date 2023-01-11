use crate::model::UserId;

#[test]
fn user_id() -> anyhow::Result<()> {
    let row_id = UserId::default();
    assert!(!row_id.get().is_nil());

    let row_id = UserId(uuid::Uuid::new_v4());
    assert!(!row_id.get().is_nil());

    let uuid_data = "85fc32fe-eaa5-46c3-b8e8-60bb658b5de7";
    let row_id: UserId = uuid_data.to_string().into();

    let new_uuid_data: String = row_id.to_string();
    assert_eq!(&new_uuid_data, uuid_data);

    Ok(())
}
