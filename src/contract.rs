#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, Uint128,
};
//use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MessagesResponse, QueryMsg};
use crate::state::{Message, CURRENT_ID, MESSAGES};

// version info for migration info
//const CONTRACT_NAME: &str = "crates.io:messages";
//const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    CURRENT_ID.save(deps.storage, &Uint128::zero().u128())?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddMessage { topic, message } => add_message(deps, info, topic, message),
    }
}

pub fn add_message(
    deps: DepsMut,
    info: MessageInfo,
    topic: String,
    message: String,
) -> Result<Response, ContractError> {
    let id = CURRENT_ID.load(deps.storage)?;

    let message = Message {
        id: Uint128::from(id),
        owner: info.sender,
        topic: topic,
        message: message,
    };

    let next_id = id.checked_add(1).unwrap();
    CURRENT_ID.save(deps.storage, &next_id)?;

    MESSAGES.save(deps.storage, message.id.u128(), &message)?;
    Ok(Response::new()
        .add_attribute("action", "execute_add_message")
        .add_attribute("message_id", message.id)
        .add_attribute("message_owner", message.owner)
        .add_attribute("message_topic", message.topic)
        .add_attribute("message_message", message.message))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCurrentId {} => to_binary(&query_current_id(deps)?),
        QueryMsg::GetAllMessage {} => to_binary(&query_all_messages(deps)?),
        QueryMsg::GetMessagesByAddr { address } => {
            to_binary(&query_messages_by_addr(deps, address)?)
        }
        QueryMsg::GetMessagesByTopic { topic } => to_binary(&query_messages_by_topic(deps, topic)?),
        QueryMsg::GetMessagesById { id } => to_binary(&query_messages_by_id(deps, id)?),
    }
}

fn query_current_id(deps: Deps) -> StdResult<Uint128> {
    Ok(Uint128::from(CURRENT_ID.load(deps.storage)?))
}

fn query_all_messages(deps: Deps) -> StdResult<MessagesResponse> {
    let messages = MESSAGES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.unwrap().1)
        .collect();
    Ok(MessagesResponse { messages })
}

fn query_messages_by_addr(deps: Deps, address: String) -> StdResult<MessagesResponse> {
    let messages = MESSAGES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.unwrap().1)
        .filter(|message| message.owner == address)
        .collect();
    Ok(MessagesResponse { messages })
}

fn query_messages_by_topic(deps: Deps, topic: String) -> StdResult<MessagesResponse> {
    let messages = MESSAGES
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| item.unwrap().1)
        .filter(|message| message.topic == topic)
        .collect();
    Ok(MessagesResponse { messages })
}

fn query_messages_by_id(deps: Deps, id: Uint128) -> StdResult<MessagesResponse> {
    let message = MESSAGES.load(deps.storage, id.u128())?;
    Ok(MessagesResponse {
        messages: vec![message],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier};
    use cosmwasm_std::{attr, from_binary, MemoryStorage, OwnedDeps};

    const ADDR1: &str = "addr1";
    const ADDR2: &str = "addr2";

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {};
        let info = mock_info(ADDR1, &[]);
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCurrentId {}).unwrap();
        let value: Uint128 = from_binary(&res).unwrap();
        assert_eq!(Uint128::zero(), value);
    }

    #[test]
    fn add_message() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        let topic = "lol".to_string();
        let message = "wut".to_string();
        let msg = ExecuteMsg::AddMessage {
            topic: topic.clone(),
            message: message.clone(),
        };
        let res = execute(deps.as_mut(), mock_env(), mock_info(ADDR1, &[]), msg).unwrap();
        assert_eq!(
            res.attributes,
            vec![
                attr("action", "execute_add_message"),
                attr("message_id", Uint128::from(0u128)),
                attr("message_owner", ADDR1),
                attr("message_topic", topic),
                attr("message_message", message),
            ]
        )
    }

    fn create_multiple_messages(deps: &mut OwnedDeps<MemoryStorage, MockApi, MockQuerier>) {
        setup_contract(deps.as_mut());
        let msg1 = ExecuteMsg::AddMessage {
            topic: "lol".to_string(),
            message: "wut".to_string(),
        };
        let msg2 = ExecuteMsg::AddMessage {
            topic: "lol".to_string(),
            message: "haha".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), mock_info(ADDR1, &[]), msg1).unwrap();
        let _res = execute(deps.as_mut(), mock_env(), mock_info(ADDR2, &[]), msg2).unwrap();
    }

    #[test]
    fn query_all_messages() {
        let mut deps = mock_dependencies();
        create_multiple_messages(&mut deps);
        let res: MessagesResponse =
            from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::GetAllMessage {}).unwrap())
                .unwrap();
        assert_eq!(res.messages.len(), 2);
    }

    #[test]
    fn query_messages_by_owner() {
        let mut deps = mock_dependencies();
        create_multiple_messages(&mut deps);
        let res: MessagesResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::GetMessagesByAddr {
                    address: ADDR1.to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(res.messages.len(), 1);
    }

    #[test]
    fn query_messages_by_id() {
        let mut deps = mock_dependencies();
        create_multiple_messages(&mut deps);
        let res: MessagesResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::GetMessagesById {
                    id: Uint128::from(1u128),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(res.messages.len(), 1);
        assert_eq!(res.messages[0].id, Uint128::from(1u128))
    }

    #[test]
    fn query_messages_by_topic() {
        let mut deps = mock_dependencies();
        create_multiple_messages(&mut deps);
        let res: MessagesResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::GetMessagesByTopic {
                    topic: "lol".to_string(),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(res.messages.len(), 2);
    }
}
