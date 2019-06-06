use std::collections::HashMap;

use futures::channel::mpsc;

use tempfile::tempdir;

use common::test_executor::TestExecutor;

use proto::app_server::messages::AppPermissions;
use timer::create_timer_incoming;

use crypto::invoice_id::{InvoiceId, INVOICE_ID_LEN};
use crypto::uid::{Uid, UID_LEN};

use crate::sim_network::create_sim_network;
use crate::utils::{
    advance_time, create_app, create_index_server, create_node, create_relay,
    named_index_server_address, named_relay_address, node_public_key, relay_address, SimDb,
};

const TIMER_CHANNEL_LEN: usize = 0;

async fn task_nodes_chain(mut test_executor: TestExecutor) {
    // Create a temporary directory.
    // Should be deleted when gets out of scope:
    let temp_dir = tempdir().unwrap();

    // Create a database manager at the temporary directory:
    let sim_db = SimDb::new(temp_dir.path().to_path_buf());

    // A network simulator:
    let sim_net_client = create_sim_network(&mut test_executor);

    // Create timer_client:
    let (mut tick_sender, tick_receiver) = mpsc::channel(TIMER_CHANNEL_LEN);
    let timer_client = create_timer_incoming(tick_receiver, test_executor.clone()).unwrap();

    let mut apps = Vec::new();

    // Create 6 nodes with apps:
    for i in 0..6 {
        sim_db.init_db(i);

        let mut trusted_apps = HashMap::new();
        trusted_apps.insert(
            i,
            AppPermissions {
                routes: true,
                buyer: true,
                seller: true,
                config: true,
            },
        );

        await!(create_node(
            i,
            sim_db.clone(),
            timer_client.clone(),
            sim_net_client.clone(),
            trusted_apps,
            test_executor.clone()
        ))
        .forget();

        apps.push(
            await!(create_app(
                i,
                sim_net_client.clone(),
                timer_client.clone(),
                i,
                test_executor.clone()
            ))
            .unwrap(),
        );
    }

    // Create relays:
    await!(create_relay(
        0,
        timer_client.clone(),
        sim_net_client.clone(),
        test_executor.clone()
    ));

    await!(create_relay(
        1,
        timer_client.clone(),
        sim_net_client.clone(),
        test_executor.clone()
    ));

    // Create three index servers:
    // 0 -- 2 -- 1
    // The only way for information to flow between the two index servers
    // is by having the middle server forward it.
    await!(create_index_server(
        2,
        timer_client.clone(),
        sim_net_client.clone(),
        vec![0, 1],
        test_executor.clone()
    ));

    await!(create_index_server(
        0,
        timer_client.clone(),
        sim_net_client.clone(),
        vec![2],
        test_executor.clone()
    ));

    await!(create_index_server(
        1,
        timer_client.clone(),
        sim_net_client.clone(),
        vec![2],
        test_executor.clone()
    ));

    // Configure relays:

    await!(apps[0].config().unwrap().add_relay(named_relay_address(0))).unwrap();
    await!(apps[0].config().unwrap().add_relay(named_relay_address(1))).unwrap();

    await!(apps[1].config().unwrap().add_relay(named_relay_address(1))).unwrap();
    await!(apps[2].config().unwrap().add_relay(named_relay_address(0))).unwrap();

    await!(apps[3].config().unwrap().add_relay(named_relay_address(1))).unwrap();
    await!(apps[3].config().unwrap().add_relay(named_relay_address(0))).unwrap();

    await!(apps[4].config().unwrap().add_relay(named_relay_address(0))).unwrap();
    await!(apps[5].config().unwrap().add_relay(named_relay_address(1))).unwrap();

    // Configure index servers:
    await!(apps[0]
        .config()
        .unwrap()
        .add_index_server(named_index_server_address(0)))
    .unwrap();
    await!(apps[0]
        .config()
        .unwrap()
        .add_index_server(named_index_server_address(2)))
    .unwrap();

    await!(apps[1]
        .config()
        .unwrap()
        .add_index_server(named_index_server_address(1)))
    .unwrap();
    await!(apps[2]
        .config()
        .unwrap()
        .add_index_server(named_index_server_address(2)))
    .unwrap();

    await!(apps[3]
        .config()
        .unwrap()
        .add_index_server(named_index_server_address(0)))
    .unwrap();

    await!(apps[4]
        .config()
        .unwrap()
        .add_index_server(named_index_server_address(1)))
    .unwrap();
    await!(apps[4]
        .config()
        .unwrap()
        .add_index_server(named_index_server_address(0)))
    .unwrap();

    await!(apps[5]
        .config()
        .unwrap()
        .add_index_server(named_index_server_address(2)))
    .unwrap();
    await!(apps[5]
        .config()
        .unwrap()
        .add_index_server(named_index_server_address(1)))
    .unwrap();

    // Wait some time:
    // await!(advance_time(40, &mut tick_sender, &test_executor));
    /*
                       5
                       |
             0 -- 1 -- 2 -- 4
                  |
                  3
    */

    // 0 --> 1
    await!(apps[0].config().unwrap().add_friend(
        node_public_key(1),
        vec![relay_address(1)],
        String::from("node1"),
        0
    ))
    .unwrap();
    await!(apps[0].config().unwrap().enable_friend(node_public_key(1))).unwrap();
    await!(apps[0].config().unwrap().open_friend(node_public_key(1))).unwrap();
    await!(apps[0]
        .config()
        .unwrap()
        .set_friend_remote_max_debt(node_public_key(1), 100))
    .unwrap();

    // 1 --> 0
    await!(apps[1].config().unwrap().add_friend(
        node_public_key(0),
        vec![relay_address(1)],
        String::from("node0"),
        0
    ))
    .unwrap();
    await!(apps[1].config().unwrap().enable_friend(node_public_key(0))).unwrap();
    await!(apps[1].config().unwrap().open_friend(node_public_key(0))).unwrap();
    await!(apps[1]
        .config()
        .unwrap()
        .set_friend_remote_max_debt(node_public_key(0), 100))
    .unwrap();

    // 1 --> 2
    await!(apps[1].config().unwrap().add_friend(
        node_public_key(2),
        vec![relay_address(0)],
        String::from("node2"),
        0
    ))
    .unwrap();
    await!(apps[1].config().unwrap().enable_friend(node_public_key(2))).unwrap();
    await!(apps[1].config().unwrap().open_friend(node_public_key(2))).unwrap();
    await!(apps[1]
        .config()
        .unwrap()
        .set_friend_remote_max_debt(node_public_key(2), 100))
    .unwrap();

    // 2 --> 1
    await!(apps[2].config().unwrap().add_friend(
        node_public_key(1),
        vec![relay_address(1)],
        String::from("node1"),
        0
    ))
    .unwrap();
    await!(apps[2].config().unwrap().enable_friend(node_public_key(1))).unwrap();
    await!(apps[2].config().unwrap().open_friend(node_public_key(1))).unwrap();
    await!(apps[2]
        .config()
        .unwrap()
        .set_friend_remote_max_debt(node_public_key(1), 100))
    .unwrap();

    // 1 --> 3
    await!(apps[1].config().unwrap().add_friend(
        node_public_key(3),
        vec![relay_address(0)],
        String::from("node3"),
        0
    ))
    .unwrap();
    await!(apps[1].config().unwrap().enable_friend(node_public_key(3))).unwrap();
    await!(apps[1].config().unwrap().open_friend(node_public_key(3))).unwrap();
    await!(apps[1]
        .config()
        .unwrap()
        .set_friend_remote_max_debt(node_public_key(3), 100))
    .unwrap();

    // 3 --> 1
    await!(apps[3].config().unwrap().add_friend(
        node_public_key(1),
        vec![relay_address(1)],
        String::from("node1"),
        0
    ))
    .unwrap();
    await!(apps[3].config().unwrap().enable_friend(node_public_key(1))).unwrap();
    await!(apps[3].config().unwrap().open_friend(node_public_key(1))).unwrap();
    await!(apps[3]
        .config()
        .unwrap()
        .set_friend_remote_max_debt(node_public_key(1), 100))
    .unwrap();

    // 2 --> 5
    await!(apps[2].config().unwrap().add_friend(
        node_public_key(5),
        vec![relay_address(1)],
        String::from("node5"),
        0
    ))
    .unwrap();
    await!(apps[2].config().unwrap().enable_friend(node_public_key(5))).unwrap();
    await!(apps[2].config().unwrap().open_friend(node_public_key(5))).unwrap();
    await!(apps[2]
        .config()
        .unwrap()
        .set_friend_remote_max_debt(node_public_key(5), 100))
    .unwrap();

    // 5 --> 2
    await!(apps[5].config().unwrap().add_friend(
        node_public_key(2),
        vec![relay_address(0)],
        String::from("node2"),
        0
    ))
    .unwrap();
    await!(apps[5].config().unwrap().enable_friend(node_public_key(2))).unwrap();
    await!(apps[5].config().unwrap().open_friend(node_public_key(2))).unwrap();
    await!(apps[5]
        .config()
        .unwrap()
        .set_friend_remote_max_debt(node_public_key(2), 100))
    .unwrap();

    // 2 --> 4
    await!(apps[2].config().unwrap().add_friend(
        node_public_key(4),
        vec![relay_address(0)],
        String::from("node4"),
        0
    ))
    .unwrap();
    await!(apps[2].config().unwrap().enable_friend(node_public_key(4))).unwrap();
    await!(apps[2].config().unwrap().open_friend(node_public_key(4))).unwrap();
    await!(apps[2]
        .config()
        .unwrap()
        .set_friend_remote_max_debt(node_public_key(4), 100))
    .unwrap();

    // 4 --> 2
    // We add the extra relay_address(1) on purpose.
    // Node4 will find out and remove it later.
    await!(apps[4].config().unwrap().add_friend(
        node_public_key(2),
        vec![relay_address(0), relay_address(1)],
        String::from("node2"),
        0
    ))
    .unwrap();
    await!(apps[4].config().unwrap().enable_friend(node_public_key(2))).unwrap();
    await!(apps[4].config().unwrap().open_friend(node_public_key(2))).unwrap();
    await!(apps[4]
        .config()
        .unwrap()
        .set_friend_remote_max_debt(node_public_key(2), 100))
    .unwrap();

    // Wait some time:
    await!(advance_time(40, &mut tick_sender, &test_executor));

    // Make sure that node1 sees node2 as online:
    let (node_report, mutations_receiver) = await!(apps[1].report().incoming_reports()).unwrap();
    drop(mutations_receiver);
    let friend_report = match node_report.funder_report.friends.get(&node_public_key(2)) {
        None => unreachable!(),
        Some(friend_report) => friend_report,
    };
    assert!(friend_report.liveness.is_online());

    /*

    // Node0: Request routes:
    let mut routes_0_4 = await!(apps[0].routes().unwrap().request_routes(
        20,
        node_public_key(0),
        node_public_key(4),
        None
    ))
    .unwrap();

    assert!(routes_0_4.len() > 0);

    // Node0: Send 10 credits to Node1:
    let chosen_route_with_capacity = routes_0_4.pop().unwrap();
    assert_eq!(chosen_route_with_capacity.capacity, 100);
    let chosen_route = chosen_route_with_capacity.route;

    let request_id = Uid::from(&[0x0; UID_LEN]);
    let invoice_id = InvoiceId::from(&[0; INVOICE_ID_LEN]);
    let dest_payment = 10;
    let receipt = await!(apps[0].send_funds().unwrap().request_send_funds(
        request_id.clone(),
        chosen_route,
        invoice_id,
        dest_payment
    ))
    .unwrap();
    await!(apps[0]
        .send_funds()
        .unwrap()
        .receipt_ack(request_id, receipt.clone()))
    .unwrap();

    // Wait some time:
    await!(advance_time(40, &mut tick_sender, &test_executor));

    // Node0: Request routes:
    let mut routes_5_3 = await!(apps[5].routes().unwrap().request_routes(
        20,
        node_public_key(5),
        node_public_key(3),
        None
    ))
    .unwrap();

    assert!(routes_5_3.len() > 0);

    // Node5: Send 10 credits to Node3:
    let chosen_route_with_capacity = routes_5_3.pop().unwrap();
    let chosen_route = chosen_route_with_capacity.route;

    let request_id = Uid::from(&[0x1; UID_LEN]);
    let invoice_id = InvoiceId::from(&[1; INVOICE_ID_LEN]);
    let dest_payment = 10;
    let receipt = await!(apps[5].send_funds().unwrap().request_send_funds(
        request_id.clone(),
        chosen_route,
        invoice_id,
        dest_payment
    ))
    .unwrap();
    await!(apps[5]
        .send_funds()
        .unwrap()
        .receipt_ack(request_id, receipt.clone()))
    .unwrap();
    */
}

#[test]
fn test_nodes_chain() {
    let test_executor = TestExecutor::new();
    let res = test_executor.run(task_nodes_chain(test_executor.clone()));
    assert!(res.is_output());
}
