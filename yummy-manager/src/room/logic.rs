/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use std::{marker::PhantomData, sync::Arc};

use anyhow::anyhow;
use actix_broker::{SystemBroker, Broker};
use serde_json::Value;
use yummy_cache::state::YummyState;
use yummy_database::DatabaseTrait;
use yummy_model::meta::collection::RoomMetaCollectionInformation;
use yummy_model::{RoomId, UserId, SendMessage};
use yummy_model::config::YummyConfig;
use yummy_model::meta::{RoomMetaType, RoomMetaAccess};
use yummy_general::database::Pool;

use super::model::RoomResponse;

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************** MACROS **************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
#[derive(Clone)]
pub struct RoomLogic<DB: DatabaseTrait + ?Sized> {
    config: Arc<YummyConfig>,
    database: Arc<Pool>,
    states: YummyState,
    _marker: PhantomData<DB>
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
impl<DB: DatabaseTrait + ?Sized> RoomLogic<DB> {
    pub fn new(config: Arc<YummyConfig>, states: YummyState, database: Arc<Pool>) -> Self {
        Self {
            config,
            database,
            states,
            _marker: PhantomData
        }
    }

    pub fn get_room_meta(&self, room_id: RoomId, key: String) -> anyhow::Result<Option<RoomMetaType>> {
        Ok(self.states.get_room_meta(&room_id, RoomMetaAccess::System)?
            .get_with_name(&key)
            .cloned()
            .map(|item| item.meta))
    }

    pub fn get_room_metas(&self, room_id: RoomId) -> anyhow::Result<Vec<RoomMetaCollectionInformation>> {
        Ok(self.states.get_room_metas(&room_id)?)
    }

    pub fn set_room_meta(&self, room_id: RoomId, key: String, value: RoomMetaType) -> anyhow::Result<()> {
        Ok(self.states.set_room_meta(&room_id, key, value)?)
    }

    pub fn remove_all_metas(&self, room_id: RoomId) -> anyhow::Result<()> {
        Ok(self.states.remove_all_room_metas(&room_id)?)
    }

    pub fn remove_room_meta(&self, room_id: RoomId, key: String) -> anyhow::Result<()> {
        Ok(self.states.remove_room_meta(&room_id, key)?)
    }

    pub fn message_to_room(&self, room_id: &RoomId, sender_user_id: Option<&UserId>, message: &Value) -> anyhow::Result<()> {
        match self.states.get_users_from_room(room_id) {
            Ok(users) => {

                /* Serialize the message */
                let message: String = RoomResponse::MessageFromRoom { user_id: sender_user_id, room_id, message }.into();

                /* System internal messages will not have UserId information */
                if let Some(sender_user_id) = sender_user_id {

                    // Discart sender from list
                    let users = users.into_iter().filter(|receiver_user| receiver_user.as_ref() != sender_user_id);
                    for receiver_user in users.into_iter() {
                        Broker::<SystemBroker>::issue_async(SendMessage {
                            message: message.clone(),
                            user_id: receiver_user
                        });
                    }
                }
                else {

                    // Send message to all users
                    for receiver_user in users.into_iter() {
                        Broker::<SystemBroker>::issue_async(SendMessage {
                            message: message.clone(),
                            user_id: receiver_user
                        });
                    }
                }

                Ok(())
            }
            Err(error) => Err(anyhow!(error))
        }
    }

    pub fn play(&self, room_id: &RoomId, sender_user_id: Option<&UserId>, message: &Value) -> anyhow::Result<()> {
        match self.states.get_users_from_room(room_id) {
            Ok(users) => {

                /* Serialize the message */
                let message: String = RoomResponse::Play { user_id: sender_user_id, room_id, message }.into();

                /* System internal messages will not have UserId information */
                if let Some(sender_user_id) = sender_user_id {

                    // Discart sender from list
                    let users = users.into_iter().filter(|receiver_user| receiver_user.as_ref() != sender_user_id);
                    for receiver_user in users.into_iter() {
                        Broker::<SystemBroker>::issue_async(SendMessage {
                            message: message.clone(),
                            user_id: receiver_user
                        });
                    }
                }
                else {

                    // Send message to all users
                    for receiver_user in users.into_iter() {
                        Broker::<SystemBroker>::issue_async(SendMessage {
                            message: message.clone(),
                            user_id: receiver_user
                        });
                    }
                }

                Ok(())
            }
            Err(error) => Err(anyhow!(error))
        }
    }

    pub fn message_to_room_user(&self, room_id: &RoomId, user_id: &UserId, sender_user_id: Option<&UserId>, message: &Value) -> anyhow::Result<()> {
        if self.states.is_user_in_room(user_id, room_id)? {

            /* Serialize the message */
            let message: String = RoomResponse::MessageFromRoom { user_id: sender_user_id, room_id, message }.into();

            Broker::<SystemBroker>::issue_async(SendMessage {
                message: message.clone(),
                user_id: Arc::new(user_id.clone())
            });

            Ok(())
        } else {
            Err(anyhow!("User could not found in room"))
        }
    }

    pub fn raw_message_to_room_user(&self, room_id: &RoomId, message: &Value) -> anyhow::Result<()> {
        match self.states.get_users_from_room(room_id) {
            Ok(users) => {

                /* Serialize the message */
                let message: String = serde_json::to_string(message)?;

                // Send message to all users
                for receiver_user in users.into_iter() {
                    Broker::<SystemBroker>::issue_async(SendMessage {
                        message: message.clone(),
                        user_id: receiver_user
                    });
                }

                Ok(())
            }
            Err(error) => Err(anyhow!(error))
        }
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
