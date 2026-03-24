use rowplus::RowPlus;
use rowplus_derive::RowPlus;

#[allow(unused)]
#[test]
fn simple() {
    #[derive(RowPlus)]
    #[rowplus(alias = "users")]
    struct UserRow {
        id: i32,
        username: String
    }

    assert_eq!(format!("{}", UserRow::columns()), "\"users\".\"id\" AS \"id\", \"users\".\"username\" AS \"username\"");
}

#[allow(unused)]
#[test]
fn flatten() {
    #[derive(RowPlus)]
    #[rowplus(alias = "users")]
    struct UserRow {
        id: i32,
        username: String
    }

    #[derive(RowPlus)]
    #[rowplus(alias = "extra_users")]
    struct ExtraUserRow {
        #[rowplus(flatten)]
        base: UserRow,
        bio: String,
    }

    assert_eq!(format!("{}", ExtraUserRow::columns()), "\"extra_users\".\"id\" AS \"id\", \"extra_users\".\"username\" AS \"username\", \"extra_users\".\"bio\" AS \"bio\"");
}

#[allow(unused)]
#[test]
fn nested() {
    #[derive(RowPlus)]
    #[rowplus(alias = "users")]
    struct UserRow {
        id: i32,
        username: String
    }

    #[derive(RowPlus)]
    #[rowplus(alias = "posts")]
    struct PostRow {
        id: i32,
        #[rowplus(nested)]
        user: UserRow,
    }

    assert_eq!(format!("{}", PostRow::columns()), "\"posts\".\"id\" AS \"id\", \"user\".\"id\" AS \"user.id\", \"user\".\"username\" AS \"user.username\"");
}

#[allow(unused)]
#[test]
fn nested_optional() {
    #[derive(RowPlus)]
    #[rowplus(alias = "users")]
    struct UserRow {
        id: i32,
        username: String
    }

    #[derive(RowPlus)]
    #[rowplus(alias = "posts")]
    struct PostRow {
        id: i32,
        #[rowplus(nested)]
        user: Option<UserRow>,
    }

    assert_eq!(format!("{}", PostRow::columns()), "\"posts\".\"id\" AS \"id\", \"user\" IS NOT NULL AS \"user$Option\", \"user\".\"id\" AS \"user.id\", \"user\".\"username\" AS \"user.username\"");
}

#[allow(unused)]
#[test]
fn nested_inside_flatten() {
    #[derive(RowPlus)]
    #[rowplus(alias = "users")]
    struct UserRow {
        id: i32,
        username: String
    }

    #[derive(RowPlus)]
    #[rowplus(alias = "posts")]
    struct PostRow {
        id: i32,
        #[rowplus(nested)]
        user: UserRow,
    }

    #[derive(RowPlus)]
    #[rowplus(alias = "extra_posts")]
    struct ExtraPostRow {
        #[rowplus(flatten)]
        base: PostRow,
        description: String,
    }

    assert_eq!(format!("{}", ExtraPostRow::columns()), "\"extra_posts\".\"id\" AS \"id\", \"user\".\"id\" AS \"user.id\", \"user\".\"username\" AS \"user.username\", \"extra_posts\".\"description\" AS \"description\"");
}
