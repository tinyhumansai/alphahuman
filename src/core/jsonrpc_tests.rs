use serde_json::json;

use super::{build_http_schema_dump, default_state, invoke_method};

#[tokio::test]
async fn invoke_health_snapshot_via_registry() {
    let result = invoke_method(default_state(), "openhuman.health_snapshot", json!({}))
        .await
        .expect("health snapshot should succeed");
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn invoke_encrypt_secret_missing_required_param_fails_validation() {
    let err = invoke_method(default_state(), "openhuman.encrypt_secret", json!({}))
        .await
        .expect_err("missing plaintext should fail");
    assert!(err.contains("missing required param 'plaintext'"));
}

#[tokio::test]
async fn invoke_doctor_models_rejects_unknown_param() {
    let err = invoke_method(
        default_state(),
        "openhuman.doctor_models",
        json!({ "invalid": true }),
    )
    .await
    .expect_err("unknown param should fail");
    assert!(err.contains("unknown param 'invalid'"));
}

#[tokio::test]
async fn invoke_config_get_runtime_flags_via_registry() {
    let result = invoke_method(
        default_state(),
        "openhuman.config_get_runtime_flags",
        json!({}),
    )
    .await
    .expect("runtime flags should succeed");
    assert!(result.get("result").is_some());
}

#[tokio::test]
async fn invoke_autocomplete_status_rejects_unknown_param() {
    let err = invoke_method(
        default_state(),
        "openhuman.autocomplete_status",
        json!({ "extra": true }),
    )
    .await
    .expect_err("unknown param should fail");
    assert!(err.contains("unknown param 'extra'"));
}

#[tokio::test]
async fn invoke_auth_store_session_missing_token_fails_validation() {
    let err = invoke_method(default_state(), "openhuman.auth_store_session", json!({}))
        .await
        .expect_err("missing token should fail");
    assert!(err.contains("missing required param 'token'"));
}

#[tokio::test]
async fn invoke_service_status_rejects_unknown_param() {
    let err = invoke_method(
        default_state(),
        "openhuman.service_status",
        json!({ "x": 1 }),
    )
    .await
    .expect_err("unknown param should fail");
    assert!(err.contains("unknown param 'x'"));
}

#[tokio::test]
async fn invoke_memory_init_accepts_empty_params() {
    // jwt_token is optional (accepted for backward compat but ignored).
    // The call may still fail for workspace reasons in test, but must NOT
    // fail with a missing-param error for jwt_token.
    let result = invoke_method(default_state(), "openhuman.memory_init", json!({})).await;
    if let Err(ref e) = result {
        assert!(
            !e.contains("missing required param") || !e.contains("jwt_token"),
            "jwt_token should be optional, got: {e}"
        );
    }
}

#[tokio::test]
async fn invoke_memory_list_namespaces_rejects_unknown_param() {
    let err = invoke_method(
        default_state(),
        "openhuman.memory_list_namespaces",
        json!({ "extra": true }),
    )
    .await
    .expect_err("unknown param should fail");
    assert!(err.contains("extra"));
}

#[tokio::test]
async fn invoke_memory_query_namespace_missing_namespace_fails() {
    let err = invoke_method(
        default_state(),
        "openhuman.memory_query_namespace",
        json!({ "query": "who owns atlas" }),
    )
    .await
    .expect_err("missing namespace should fail");
    assert!(err.contains("namespace"));
}

#[tokio::test]
async fn invoke_memory_recall_memories_rejects_unknown_param() {
    let err = invoke_method(
        default_state(),
        "openhuman.memory_recall_memories",
        json!({ "namespace": "team", "extra": true }),
    )
    .await
    .expect_err("unknown param should fail");
    assert!(err.contains("extra"));
}

#[tokio::test]
async fn invoke_migrate_openclaw_rejects_unknown_param() {
    let err = invoke_method(
        default_state(),
        "openhuman.migrate_openclaw",
        json!({ "x": 1 }),
    )
    .await
    .expect_err("unknown param should fail");
    assert!(err.contains("unknown param 'x'"));
}

#[tokio::test]
async fn invoke_local_ai_download_asset_missing_required_param_fails_validation() {
    let err = invoke_method(
        default_state(),
        "openhuman.local_ai_download_asset",
        json!({}),
    )
    .await
    .expect_err("missing capability should fail");
    assert!(err.contains("missing required param 'capability'"));
}

#[test]
fn http_schema_dump_includes_openhuman_and_core_methods() {
    let dump = build_http_schema_dump();
    let methods = dump.methods;
    assert!(
        methods
            .iter()
            .any(|m| m.method == "core.version" && m.namespace == "core"),
        "schema dump should include core methods"
    );

    assert!(
        methods
            .iter()
            .any(|m| m.method == "openhuman.health_snapshot"),
        "schema dump should include migrated openhuman methods"
    );

    assert!(
        methods
            .iter()
            .any(|m| m.method == "openhuman.billing_get_current_plan"),
        "schema dump should include billing methods"
    );

    assert!(
        methods
            .iter()
            .any(|m| m.method == "openhuman.team_list_members"),
        "schema dump should include team methods"
    );
}

#[tokio::test]
async fn billing_get_current_plan_rejects_unknown_param() {
    let err = invoke_method(
        default_state(),
        "openhuman.billing_get_current_plan",
        json!({ "extra": true }),
    )
    .await
    .expect_err("unknown param should fail");
    assert!(err.contains("unknown param 'extra'"));
}

#[tokio::test]
async fn billing_purchase_plan_missing_plan_fails_validation() {
    let err = invoke_method(
        default_state(),
        "openhuman.billing_purchase_plan",
        json!({}),
    )
    .await
    .expect_err("missing plan should fail");
    assert!(err.contains("missing required param 'plan'"));
}

#[tokio::test]
async fn billing_top_up_missing_amount_fails_validation() {
    let err = invoke_method(default_state(), "openhuman.billing_top_up", json!({}))
        .await
        .expect_err("missing amountUsd should fail");
    assert!(err.contains("missing required param 'amountUsd'"));
}

#[tokio::test]
async fn billing_top_up_rejects_unknown_param() {
    let err = invoke_method(
        default_state(),
        "openhuman.billing_top_up",
        json!({ "amountUsd": 10.0, "unknownField": true }),
    )
    .await
    .expect_err("unknown param should fail");
    assert!(err.contains("unknown param 'unknownField'"));
}

#[tokio::test]
async fn billing_create_portal_session_rejects_unknown_param() {
    let err = invoke_method(
        default_state(),
        "openhuman.billing_create_portal_session",
        json!({ "x": 1 }),
    )
    .await
    .expect_err("unknown param should fail");
    assert!(err.contains("unknown param 'x'"));
}

#[tokio::test]
async fn team_list_members_missing_team_id_fails_validation() {
    let err = invoke_method(default_state(), "openhuman.team_list_members", json!({}))
        .await
        .expect_err("missing teamId should fail");
    assert!(err.contains("missing required param 'teamId'"));
}

#[tokio::test]
async fn team_list_members_rejects_unknown_param() {
    let err = invoke_method(
        default_state(),
        "openhuman.team_list_members",
        json!({ "teamId": "t1", "extra": true }),
    )
    .await
    .expect_err("unknown param should fail");
    assert!(err.contains("unknown param 'extra'"));
}

#[tokio::test]
async fn team_create_invite_missing_team_id_fails_validation() {
    let err = invoke_method(default_state(), "openhuman.team_create_invite", json!({}))
        .await
        .expect_err("missing teamId should fail");
    assert!(err.contains("missing required param 'teamId'"));
}

#[tokio::test]
async fn team_remove_member_missing_required_params_fails_validation() {
    let err = invoke_method(
        default_state(),
        "openhuman.team_remove_member",
        json!({ "teamId": "t1" }),
    )
    .await
    .expect_err("missing userId should fail");
    assert!(err.contains("missing required param 'userId'"));
}

#[tokio::test]
async fn team_change_member_role_missing_role_fails_validation() {
    let err = invoke_method(
        default_state(),
        "openhuman.team_change_member_role",
        json!({ "teamId": "t1", "userId": "u1" }),
    )
    .await
    .expect_err("missing role should fail");
    assert!(err.contains("missing required param 'role'"));
}

#[tokio::test]
async fn billing_create_coinbase_charge_missing_plan_fails_validation() {
    let err = invoke_method(
        default_state(),
        "openhuman.billing_create_coinbase_charge",
        json!({}),
    )
    .await
    .expect_err("missing plan should fail");
    assert!(err.contains("missing required param 'plan'"));
}

#[tokio::test]
async fn billing_create_coinbase_charge_rejects_unknown_param() {
    let err = invoke_method(
        default_state(),
        "openhuman.billing_create_coinbase_charge",
        json!({ "plan": "pro", "extra": true }),
    )
    .await
    .expect_err("unknown param should fail");
    assert!(err.contains("unknown param 'extra'"));
}

#[tokio::test]
async fn team_list_invites_missing_team_id_fails_validation() {
    let err = invoke_method(default_state(), "openhuman.team_list_invites", json!({}))
        .await
        .expect_err("missing teamId should fail");
    assert!(err.contains("missing required param 'teamId'"));
}

#[tokio::test]
async fn team_list_invites_rejects_unknown_param() {
    let err = invoke_method(
        default_state(),
        "openhuman.team_list_invites",
        json!({ "teamId": "t1", "extra": true }),
    )
    .await
    .expect_err("unknown param should fail");
    assert!(err.contains("unknown param 'extra'"));
}

#[tokio::test]
async fn team_revoke_invite_missing_team_id_fails_validation() {
    let err = invoke_method(default_state(), "openhuman.team_revoke_invite", json!({}))
        .await
        .expect_err("missing teamId should fail");
    assert!(err.contains("missing required param 'teamId'"));
}

#[tokio::test]
async fn team_revoke_invite_missing_invite_id_fails_validation() {
    let err = invoke_method(
        default_state(),
        "openhuman.team_revoke_invite",
        json!({ "teamId": "t1" }),
    )
    .await
    .expect_err("missing inviteId should fail");
    assert!(err.contains("missing required param 'inviteId'"));
}

#[tokio::test]
async fn schema_dump_includes_new_billing_and_team_methods() {
    let dump = build_http_schema_dump();
    let methods: Vec<&str> = dump.methods.iter().map(|m| m.method.as_str()).collect();
    for expected in &[
        "openhuman.billing_get_current_plan",
        "openhuman.billing_purchase_plan",
        "openhuman.billing_create_portal_session",
        "openhuman.billing_top_up",
        "openhuman.billing_create_coinbase_charge",
        "openhuman.team_list_members",
        "openhuman.team_create_invite",
        "openhuman.team_list_invites",
        "openhuman.team_revoke_invite",
        "openhuman.team_remove_member",
        "openhuman.team_change_member_role",
    ] {
        assert!(
            methods.contains(expected),
            "schema dump missing expected method: {expected}"
        );
    }
}
