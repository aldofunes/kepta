mod common;

use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn create_household_sets_owner(db: PgPool) {
    let (user, _) = common::insert_user(&db).await;
    let household = common::insert_household(&db, user.id).await;

    assert_eq!(household.name, "Test Home");
    assert_eq!(household.year_built, Some(1990));

    let role = kepta_db::households::get_role(&db, household.id, user.id)
        .await
        .unwrap();
    assert_eq!(role, Some(kepta_types::households::MemberRole::Owner));
}

#[sqlx::test(migrations = "./migrations")]
async fn find_by_user_returns_owned_households(db: PgPool) {
    let (user, _) = common::insert_user(&db).await;
    let h1 = common::insert_household(&db, user.id).await;
    let h2 = common::insert_household(&db, user.id).await;

    let result = kepta_db::households::find_by_user(&db, user.id)
        .await
        .unwrap();
    let ids: Vec<_> = result.iter().map(|h| h.id).collect();
    assert!(ids.contains(&h1.id));
    assert!(ids.contains(&h2.id));
}

#[sqlx::test(migrations = "./migrations")]
async fn find_by_user_excludes_other_users_households(db: PgPool) {
    let (user_a, _) = common::insert_user(&db).await;
    let (user_b, _) = common::insert_user(&db).await;
    common::insert_household(&db, user_a.id).await;

    let result = kepta_db::households::find_by_user(&db, user_b.id)
        .await
        .unwrap();
    assert!(result.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn is_member_true_for_owner(db: PgPool) {
    let (user, _) = common::insert_user(&db).await;
    let h = common::insert_household(&db, user.id).await;
    assert!(kepta_db::households::is_member(&db, h.id, user.id).await.unwrap());
}

#[sqlx::test(migrations = "./migrations")]
async fn is_member_true_for_invited_member(db: PgPool) {
    let (owner, _) = common::insert_user(&db).await;
    let (member, _) = common::insert_user(&db).await;
    let h = common::insert_household(&db, owner.id).await;
    common::invite_and_join(&db, h.id, member.id).await;

    assert!(kepta_db::households::is_member(&db, h.id, member.id).await.unwrap());
}

#[sqlx::test(migrations = "./migrations")]
async fn is_member_false_for_stranger(db: PgPool) {
    let (owner, _) = common::insert_user(&db).await;
    let (stranger, _) = common::insert_user(&db).await;
    let h = common::insert_household(&db, owner.id).await;

    assert!(!kepta_db::households::is_member(&db, h.id, stranger.id).await.unwrap());
}

#[sqlx::test(migrations = "./migrations")]
async fn count_by_user_increments_per_household(db: PgPool) {
    let (user, _) = common::insert_user(&db).await;
    assert_eq!(kepta_db::households::count_by_user(&db, user.id).await.unwrap(), 0);

    common::insert_household(&db, user.id).await;
    assert_eq!(kepta_db::households::count_by_user(&db, user.id).await.unwrap(), 1);

    common::insert_household(&db, user.id).await;
    assert_eq!(kepta_db::households::count_by_user(&db, user.id).await.unwrap(), 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn get_role_returns_member_role(db: PgPool) {
    let (owner, _) = common::insert_user(&db).await;
    let (member_user, _) = common::insert_user(&db).await;
    let h = common::insert_household(&db, owner.id).await;
    common::invite_and_join(&db, h.id, member_user.id).await;

    let role = kepta_db::households::get_role(&db, h.id, member_user.id)
        .await
        .unwrap();
    assert_eq!(role, Some(kepta_types::households::MemberRole::Member));
}

#[sqlx::test(migrations = "./migrations")]
async fn get_role_returns_none_for_non_member(db: PgPool) {
    let (owner, _) = common::insert_user(&db).await;
    let (stranger, _) = common::insert_user(&db).await;
    let h = common::insert_household(&db, owner.id).await;

    let role = kepta_db::households::get_role(&db, h.id, stranger.id)
        .await
        .unwrap();
    assert!(role.is_none());
}
