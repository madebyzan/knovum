use crate::{
    agent_runtime::protocol::control::AgentExecutionIdentity,
    spaces::conversations::builtin_agent::identity::TaskIds,
};

pub(in crate::spaces::conversations) async fn database() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::raw_sql("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1);
        INSERT INTO spaces_authorization_grants(id,principal_kind,principal_id,scope_kind,home_space_id,read_space_ids,write_space_ids,action_ids,valid_until,created_at)
        VALUES('grant','builtin_agent','task','space','space','[\"space\"]','[\"space\"]','[]',9999999999999,1);
        INSERT INTO spaces_conversations(id,space_id,title,created_at,updated_at) VALUES('conversation','space','Test',1,1);
        INSERT INTO spaces_builtin_agent_execution_segments(id,conversation_id,branch_id,execution_revision,graph_version,frozen_target,model_profile,target_selection,catalog_digest,created_at)
        VALUES('thread','conversation','branch','revision','test','{}','{}','{}','aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',1);
        INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at)
        VALUES('run','space','space','builtin_agent','task','builtin_agent','running','{}','{}','{\"maxTurns\":8,\"maxTools\":4,\"maxTotalTokens\":1000,\"maxDurationSeconds\":900}',1,1);
        INSERT INTO spaces_builtin_agent_tasks(id,run_id,conversation_id,branch_id,grant_id,model_binding,catalog_digest,segment_id,input_id,created_at,updated_at)
        VALUES('task','run','conversation','branch','grant','{}','aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa','thread','input:task',1,1);")
        .execute(&pool).await.unwrap();
    sqlx::query("UPDATE spaces_runs SET created_at=$1 WHERE id='run'")
        .bind(crate::clock::unix_time_millis().unwrap())
        .execute(&pool)
        .await
        .unwrap();
    pool
}

pub(super) fn identity() -> AgentExecutionIdentity {
    AgentExecutionIdentity {
        run_id: "run".into(),
        thread_id: "thread".into(),
        execution_revision: "revision".into(),
        control_epoch: 0,
    }
}

pub(in crate::spaces::conversations) fn ids() -> TaskIds {
    TaskIds {
        task_id: "task".into(),
        run_id: "run".into(),
        conversation_id: "conversation".into(),
        branch_id: "branch".into(),
        grant_id: "grant".into(),
    }
}
