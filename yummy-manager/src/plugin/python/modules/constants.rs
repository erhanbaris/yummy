#[rustpython::vm::pymodule]
pub mod _constants {
    /* **************************************************************************************************************** */
    /* **************************************************** MODS ****************************************************** */
    /* *************************************************** IMPORTS **************************************************** */
    /* **************************************************************************************************************** */
    use yummy_model::{meta::{UserMetaAccess, MetaAction}, UserType, CreateRoomAccessType, RoomUserType};

    /* **************************************************************************************************************** */
    /* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
    /* **************************************************************************************************************** */
    /* UserType */
    #[pyattr]
    const USER_TYPE_USER: u32 = UserType::User as u32;
    #[pyattr]
    const USER_TYPE_MOD: u32 = UserType::Mod as u32;
    #[pyattr]
    const USER_TYPE_ADMIN: u32 = UserType::Admin as u32;


    /* UserMetaAccess */
    #[pyattr]
    const USER_META_ACCESS_ANONYMOUS: u32 = UserMetaAccess::Anonymous as u32;
    #[pyattr]
    const USER_META_ACCESS_USER: u32 = UserMetaAccess::User as u32;
    #[pyattr]
    const USER_META_ACCESS_FRIEND: u32 = UserMetaAccess::Friend as u32;
    #[pyattr]
    const USER_META_ACCESS_ME: u32 = UserMetaAccess::Me as u32;
    #[pyattr]
    const USER_META_ACCESS_MOD: u32 = UserMetaAccess::Mod as u32;
    #[pyattr]
    const USER_META_ACCESS_ADMIN: u32 = UserMetaAccess::Admin as u32;
    #[pyattr]
    const USER_META_ACCESS_SYSTEM: u32 = UserMetaAccess::System as u32;


    /* MetaAction */
    #[pyattr]
    const META_ACTION_ONLY_ADD_OR_UPDATE: u32 = MetaAction::OnlyAddOrUpdate as u32;
    #[pyattr]
    const META_ACTION_REMOVE_UNUSED_METAS: u32 = MetaAction::RemoveUnusedMetas as u32;
    #[pyattr]
    const META_ACTION_REMOVE_ALL_METAS: u32 = MetaAction::RemoveAllMetas as u32;

    /* CreateRoomAccessType */
    #[pyattr]
    const ROOM_ACCESS_TYPE_PUBLIC: u32 = CreateRoomAccessType::Public as u32;
    #[pyattr]
    const ROOM_ACCESS_TYPE_PRIVATE: u32 = CreateRoomAccessType::Private as u32;
    #[pyattr]
    const ROOM_ACCESS_TYPE_FRIEND: u32 = CreateRoomAccessType::Friend as u32;

    /* RoomUserType */
    #[pyattr]
    const ROOM_USER_TYPE_USER: u32 = RoomUserType::User as u32;
    #[pyattr]
    const ROOM_USER_TYPE_MODERATOR: u32 = RoomUserType::Moderator as u32;
    #[pyattr]
    const ROOM_USER_TYPE_OWNER: u32 = RoomUserType::Owner as u32;

    #[pyattr]
    const ROOM_INFO_TYPE_ROOM_NAME: u32 = 0; // RoomInfoTypeVariant::RoomName;
    #[pyattr]
    const ROOM_INFO_TYPE_DESCRIPTION: u32 = 1; // RoomInfoTypeVariant::Description;
    #[pyattr]
    const ROOM_INFO_TYPE_USERS: u32 = 2; // RoomInfoTypeVariant::Users;
    #[pyattr]
    const ROOM_INFO_TYPE_MAX_USER: u32 = 3; // RoomInfoTypeVariant::MaxUser;
    #[pyattr]
    const ROOM_INFO_TYPE_USER_LENGTH: u32 = 4; // RoomInfoTypeVariant::UserLength;
    #[pyattr]
    const ROOM_INFO_TYPE_ACCESS_TYPE: u32 = 5; // RoomInfoTypeVariant::AccessType;
    #[pyattr]
    const ROOM_INFO_TYPE_TAGS: u32 = 6; // RoomInfoTypeVariant::Tags;
    #[pyattr]
    const ROOM_INFO_TYPE_METAS: u32 = 7; // RoomInfoTypeVariant::Metas;
    #[pyattr]
    const ROOM_INFO_TYPE_INSERT_DATE: u32 = 8; // RoomInfoTypeVariant::InsertDate;
    #[pyattr]
    const ROOM_INFO_TYPE_JOIN_REQUEST: u32 = 9; // RoomInfoTypeVariant::JoinRequest;
    #[pyattr]
    const ROOM_INFO_TYPE_BANNED_USERS: u32 = 10; // RoomInfoTypeVariant::BannedUsers;

    /* **************************************************************************************************************** */
    /* **************************************************** MACROS **************************************************** */
    /* *************************************************** STRUCTS **************************************************** */
    /* **************************************************** ENUMS ***************************************************** */
    /* ************************************************** FUNCTIONS *************************************************** */
    /* *************************************************** TRAITS ***************************************************** */
    /* ************************************************* IMPLEMENTS *************************************************** */
    /* ********************************************** TRAIT IMPLEMENTS ************************************************ */
    /* ************************************************* MACROS CALL ************************************************** */
    /* ************************************************** UNIT TESTS ************************************************** */
    /* **************************************************************************************************************** */
}
