use yummy_model::meta::{RoomMetaAccess, MetaType, UserMetaAccess};

use serde_json;

#[test]
fn basic_deserialization() {
    // User meta data
    let s = "2";
    let deserialized: MetaType<UserMetaAccess> = serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::Number(2.0, UserMetaAccess::Anonymous));

    let s = "\"erhan\"";
    let deserialized: MetaType<UserMetaAccess> = serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::String("erhan".to_string(), UserMetaAccess::Anonymous));

    let s = "true";
    let deserialized: MetaType<UserMetaAccess> = serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::Bool(true, UserMetaAccess::Anonymous));

    let s = "false";
    let deserialized: MetaType<UserMetaAccess> = serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::Bool(false, UserMetaAccess::Anonymous));

    let s = "10.5";
    let deserialized: MetaType<UserMetaAccess>= serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::Number(10.5, UserMetaAccess::Anonymous));

    let s = "null";
    let deserialized: MetaType<UserMetaAccess>= serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::Null);

    // Room meta data
    let s = "2";
    let deserialized: MetaType<RoomMetaAccess> = serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::Number(2.0, RoomMetaAccess::Anonymous));

    let s = "\"erhan\"";
    let deserialized: MetaType<RoomMetaAccess> = serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::String("erhan".to_string(), RoomMetaAccess::Anonymous));

    let s = "true";
    let deserialized: MetaType<RoomMetaAccess> = serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::Bool(true, RoomMetaAccess::Anonymous));

    let s = "false";
    let deserialized: MetaType<RoomMetaAccess> = serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::Bool(false, RoomMetaAccess::Anonymous));

    let s = "10.5";
    let deserialized: MetaType<RoomMetaAccess>= serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::Number(10.5, RoomMetaAccess::Anonymous));

    let s = "null";
    let deserialized: MetaType<RoomMetaAccess>= serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::Null);

    let s = "[]";
    let deserialized: MetaType<RoomMetaAccess>= serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::List(Box::new(Vec::new()), RoomMetaAccess::Anonymous));

    let s = "[1,2,3,4]";
    let deserialized: MetaType<RoomMetaAccess>= serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::List(Box::new(vec![MetaType::Number(1.0, RoomMetaAccess::Anonymous), MetaType::Number(2.0, RoomMetaAccess::Anonymous), MetaType::Number(3.0, RoomMetaAccess::Anonymous), MetaType::Number(4.0, RoomMetaAccess::Anonymous)]), RoomMetaAccess::Anonymous));
}

#[test]
fn dict_deserialization() {
    // User meta data
    let s = r#"{"access": 3, "value": true}"#;
    let deserialized: MetaType<UserMetaAccess>= serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::Bool(true, UserMetaAccess::Me));

    let s = r#"{"access": 4, "value": "erhan"}"#;
    let deserialized: MetaType<UserMetaAccess>= serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::String("erhan".to_string(), UserMetaAccess::Mod));

    let s = r#"{"access": 0, "value": true}"#;
    let deserialized: MetaType<UserMetaAccess>= serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, serde_json::from_str("true").unwrap());

    // Room meta data
    let s = r#"{"access": 3, "value": true}"#;
    let deserialized: MetaType<RoomMetaAccess>= serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::Bool(true, RoomMetaAccess::Owner));

    let s = r#"{"access": 4, "value": "erhan"}"#;
    let deserialized: MetaType<RoomMetaAccess>= serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, MetaType::String("erhan".to_string(), RoomMetaAccess::Admin));

    let s = r#"{"access": 0, "value": true}"#;
    let deserialized: MetaType<RoomMetaAccess>= serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, serde_json::from_str("true").unwrap());
    
    let s = r#"{"access": 0, "value": [1,2,3]}"#;
    let deserialized: MetaType<RoomMetaAccess>= serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, serde_json::from_str("[1,2,3]").unwrap());
    
    let s = r#"{"access": 0, "value": [[1,2,3],2,3]}"#;
    let deserialized: MetaType<RoomMetaAccess>= serde_json::from_str(s).unwrap();
    assert_eq!(deserialized, serde_json::from_str("[[1,2,3],2,3]").unwrap());
}

#[test]
fn wrong_deserialization() {
    // User meta data
    let s = r#"{"access": 3}"#;
    assert_eq!(serde_json::from_str::<MetaType<UserMetaAccess>>(s).err().unwrap().to_string(), r#""value" key is missing at line 1 column 13"#);

    let s = r#"{"value": true}"#;
    assert_eq!(serde_json::from_str::<MetaType<UserMetaAccess>>(s).err().unwrap().to_string(), r#""access" key is missing at line 1 column 15"#);

    let s = r#"{"access": 0, "value": true, "wrong": 1}"#;
    assert_eq!(serde_json::from_str::<MetaType<UserMetaAccess>>(s).err().unwrap().to_string(), r#""wrong" is not valid at line 1 column 36"#);

    let s = r#"{"access": "0", "value": true}"#;
    assert_eq!(serde_json::from_str::<MetaType<UserMetaAccess>>(s).err().unwrap().to_string(), r#"Invalid "access" type at line 1 column 14"#);

    let s = r#"{"access": 0, "value": {}}"#;
    assert_eq!(serde_json::from_str::<MetaType<UserMetaAccess>>(s).err().unwrap().to_string(), r#""access" key is missing at line 1 column 25"#);


    // Room meta data
    let s = r#"{"access": 3}"#;
    assert_eq!(serde_json::from_str::<MetaType<RoomMetaAccess>>(s).err().unwrap().to_string(), r#""value" key is missing at line 1 column 13"#);

    let s = r#"{"value": true}"#;
    assert_eq!(serde_json::from_str::<MetaType<RoomMetaAccess>>(s).err().unwrap().to_string(), r#""access" key is missing at line 1 column 15"#);

    let s = r#"{"access": 0, "value": true, "wrong": 1}"#;
    assert_eq!(serde_json::from_str::<MetaType<RoomMetaAccess>>(s).err().unwrap().to_string(), r#""wrong" is not valid at line 1 column 36"#);

    let s = r#"{"access": "0", "value": true}"#;
    assert_eq!(serde_json::from_str::<MetaType<RoomMetaAccess>>(s).err().unwrap().to_string(), r#"Invalid "access" type at line 1 column 14"#);

    let s = r#"{"access": 0, "value": {}}"#;
    assert_eq!(serde_json::from_str::<MetaType<RoomMetaAccess>>(s).err().unwrap().to_string(), r#""access" key is missing at line 1 column 25"#);
}