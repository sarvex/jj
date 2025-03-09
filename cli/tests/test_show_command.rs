// Copyright 2022 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use regex::Regex;

use crate::common::TestEnvironment;

#[test]
fn test_show() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    let output = test_env.run_jj_in(&repo_path, ["show"]);
    let output = output.normalize_stdout_with(|s| s.split_inclusive('\n').skip(2).collect());

    insta::assert_snapshot!(output, @r"
    Author   : Test User <test.user@example.com> (2001-02-03 08:05:07)
    Committer: Test User <test.user@example.com> (2001-02-03 08:05:07)

        (no description set)
    ");
}

#[test]
fn test_show_basic() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "foo\n").unwrap();
    std::fs::write(repo_path.join("file2"), "foo\nbaz qux\n").unwrap();
    test_env.run_jj_in(&repo_path, ["new"]).success();
    std::fs::remove_file(repo_path.join("file1")).unwrap();
    std::fs::write(repo_path.join("file2"), "foo\nbar\nbaz quux\n").unwrap();
    std::fs::write(repo_path.join("file3"), "foo\n").unwrap();

    let output = test_env.run_jj_in(&repo_path, ["show"]);
    insta::assert_snapshot!(output, @r"
    Commit ID: e34f04317a81edc6ba41fef239c0d0180f10656f
    Change ID: rlvkpnrzqnoowoytxnquwvuryrwnrmlp
    Author   : Test User <test.user@example.com> (2001-02-03 08:05:09)
    Committer: Test User <test.user@example.com> (2001-02-03 08:05:09)

        (no description set)

    Modified regular file file2:
       1    1: foo
            2: bar
       2    3: baz quxquux
    Modified regular file file3 (file1 => file3):
    ");

    let output = test_env.run_jj_in(&repo_path, ["show", "--context=0"]);
    insta::assert_snapshot!(output, @r"
    Commit ID: e34f04317a81edc6ba41fef239c0d0180f10656f
    Change ID: rlvkpnrzqnoowoytxnquwvuryrwnrmlp
    Author   : Test User <test.user@example.com> (2001-02-03 08:05:09)
    Committer: Test User <test.user@example.com> (2001-02-03 08:05:09)

        (no description set)

    Modified regular file file2:
       1    1: foo
            2: bar
       2    3: baz quxquux
    Modified regular file file3 (file1 => file3):
    ");

    let output = test_env.run_jj_in(&repo_path, ["show", "--color=debug"]);
    insta::assert_snapshot!(output, @"Commit ID: \u{1b}[38;5;4m<<commit_id::e34f04317a81edc6ba41fef239c0d0180f10656f>>\u{1b}[39m\nChange ID: \u{1b}[38;5;5m<<change_id::rlvkpnrzqnoowoytxnquwvuryrwnrmlp>>\u{1b}[39m\nAuthor   : \u{1b}[38;5;3m<<author name::Test User>>\u{1b}[39m <\u{1b}[38;5;3m<<author email local::test.user>><<author email::@>><<author email domain::example.com>>\u{1b}[39m> (\u{1b}[38;5;6m<<author timestamp local format::2001-02-03 08:05:09>>\u{1b}[39m)\nCommitter: \u{1b}[38;5;3m<<committer name::Test User>>\u{1b}[39m <\u{1b}[38;5;3m<<committer email local::test.user>><<committer email::@>><<committer email domain::example.com>>\u{1b}[39m> (\u{1b}[38;5;6m<<committer timestamp local format::2001-02-03 08:05:09>>\u{1b}[39m)\n\n\u{1b}[38;5;3m<<description placeholder::    (no description set)>>\u{1b}[39m\n\n\u{1b}[38;5;3m<<diff header::Modified regular file file2:>>\u{1b}[39m\n\u{1b}[38;5;1m<<diff removed line_number::   1>>\u{1b}[39m<<diff:: >>\u{1b}[38;5;2m<<diff added line_number::   1>>\u{1b}[39m<<diff::: foo>>\n<<diff::     >>\u{1b}[38;5;2m<<diff added line_number::   2>>\u{1b}[39m<<diff::: >>\u{1b}[4m\u{1b}[38;5;2m<<diff added token::bar>>\u{1b}[24m\u{1b}[39m\n\u{1b}[38;5;1m<<diff removed line_number::   2>>\u{1b}[39m<<diff:: >>\u{1b}[38;5;2m<<diff added line_number::   3>>\u{1b}[39m<<diff::: baz >>\u{1b}[4m\u{1b}[38;5;1m<<diff removed token::qux>>\u{1b}[38;5;2m<<diff added token::quux>>\u{1b}[24m\u{1b}[39m<<diff::>>\n\u{1b}[38;5;3m<<diff header::Modified regular file file3 (file1 => file3):>>\u{1b}[39m");

    let output = test_env.run_jj_in(&repo_path, ["show", "-s"]);
    insta::assert_snapshot!(output, @r"
    Commit ID: e34f04317a81edc6ba41fef239c0d0180f10656f
    Change ID: rlvkpnrzqnoowoytxnquwvuryrwnrmlp
    Author   : Test User <test.user@example.com> (2001-02-03 08:05:09)
    Committer: Test User <test.user@example.com> (2001-02-03 08:05:09)

        (no description set)

    M file2
    R {file1 => file3}
    ");

    let output = test_env.run_jj_in(&repo_path, ["show", "--types"]);
    insta::assert_snapshot!(output, @r"
    Commit ID: e34f04317a81edc6ba41fef239c0d0180f10656f
    Change ID: rlvkpnrzqnoowoytxnquwvuryrwnrmlp
    Author   : Test User <test.user@example.com> (2001-02-03 08:05:09)
    Committer: Test User <test.user@example.com> (2001-02-03 08:05:09)

        (no description set)

    FF file2
    FF {file1 => file3}
    ");

    let output = test_env.run_jj_in(&repo_path, ["show", "--git"]);
    insta::assert_snapshot!(output, @r"
    Commit ID: e34f04317a81edc6ba41fef239c0d0180f10656f
    Change ID: rlvkpnrzqnoowoytxnquwvuryrwnrmlp
    Author   : Test User <test.user@example.com> (2001-02-03 08:05:09)
    Committer: Test User <test.user@example.com> (2001-02-03 08:05:09)

        (no description set)

    diff --git a/file2 b/file2
    index 523a4a9de8..485b56a572 100644
    --- a/file2
    +++ b/file2
    @@ -1,2 +1,3 @@
     foo
    -baz qux
    +bar
    +baz quux
    diff --git a/file1 b/file3
    rename from file1
    rename to file3
    ");

    let output = test_env.run_jj_in(&repo_path, ["show", "--git", "--context=0"]);
    insta::assert_snapshot!(output, @r"
    Commit ID: e34f04317a81edc6ba41fef239c0d0180f10656f
    Change ID: rlvkpnrzqnoowoytxnquwvuryrwnrmlp
    Author   : Test User <test.user@example.com> (2001-02-03 08:05:09)
    Committer: Test User <test.user@example.com> (2001-02-03 08:05:09)

        (no description set)

    diff --git a/file2 b/file2
    index 523a4a9de8..485b56a572 100644
    --- a/file2
    +++ b/file2
    @@ -2,1 +2,2 @@
    -baz qux
    +bar
    +baz quux
    diff --git a/file1 b/file3
    rename from file1
    rename to file3
    ");

    let output = test_env.run_jj_in(&repo_path, ["show", "--git", "--color=debug"]);
    insta::assert_snapshot!(output, @"Commit ID: \u{1b}[38;5;4m<<commit_id::e34f04317a81edc6ba41fef239c0d0180f10656f>>\u{1b}[39m\nChange ID: \u{1b}[38;5;5m<<change_id::rlvkpnrzqnoowoytxnquwvuryrwnrmlp>>\u{1b}[39m\nAuthor   : \u{1b}[38;5;3m<<author name::Test User>>\u{1b}[39m <\u{1b}[38;5;3m<<author email local::test.user>><<author email::@>><<author email domain::example.com>>\u{1b}[39m> (\u{1b}[38;5;6m<<author timestamp local format::2001-02-03 08:05:09>>\u{1b}[39m)\nCommitter: \u{1b}[38;5;3m<<committer name::Test User>>\u{1b}[39m <\u{1b}[38;5;3m<<committer email local::test.user>><<committer email::@>><<committer email domain::example.com>>\u{1b}[39m> (\u{1b}[38;5;6m<<committer timestamp local format::2001-02-03 08:05:09>>\u{1b}[39m)\n\n\u{1b}[38;5;3m<<description placeholder::    (no description set)>>\u{1b}[39m\n\n\u{1b}[1m<<diff file_header::diff --git a/file2 b/file2>>\u{1b}[0m\n\u{1b}[1m<<diff file_header::index 523a4a9de8..485b56a572 100644>>\u{1b}[0m\n\u{1b}[1m<<diff file_header::--- a/file2>>\u{1b}[0m\n\u{1b}[1m<<diff file_header::+++ b/file2>>\u{1b}[0m\n\u{1b}[38;5;6m<<diff hunk_header::@@ -1,2 +1,3 @@>>\u{1b}[39m\n<<diff context:: foo>>\n\u{1b}[38;5;1m<<diff removed::-baz >>\u{1b}[4m<<diff removed token::qux>>\u{1b}[24m<<diff removed::>>\u{1b}[39m\n\u{1b}[38;5;2m<<diff added::+>>\u{1b}[4m<<diff added token::bar>>\u{1b}[24m\u{1b}[39m\n\u{1b}[38;5;2m<<diff added::+baz >>\u{1b}[4m<<diff added token::quux>>\u{1b}[24m<<diff added::>>\u{1b}[39m\n\u{1b}[1m<<diff file_header::diff --git a/file1 b/file3>>\u{1b}[0m\n\u{1b}[1m<<diff file_header::rename from file1>>\u{1b}[0m\n\u{1b}[1m<<diff file_header::rename to file3>>\u{1b}[0m");

    let output = test_env.run_jj_in(&repo_path, ["show", "-s", "--git"]);
    insta::assert_snapshot!(output, @r"
    Commit ID: e34f04317a81edc6ba41fef239c0d0180f10656f
    Change ID: rlvkpnrzqnoowoytxnquwvuryrwnrmlp
    Author   : Test User <test.user@example.com> (2001-02-03 08:05:09)
    Committer: Test User <test.user@example.com> (2001-02-03 08:05:09)

        (no description set)

    M file2
    R {file1 => file3}
    diff --git a/file2 b/file2
    index 523a4a9de8..485b56a572 100644
    --- a/file2
    +++ b/file2
    @@ -1,2 +1,3 @@
     foo
    -baz qux
    +bar
    +baz quux
    diff --git a/file1 b/file3
    rename from file1
    rename to file3
    ");

    let output = test_env.run_jj_in(&repo_path, ["show", "--stat"]);
    insta::assert_snapshot!(output, @r"
    Commit ID: e34f04317a81edc6ba41fef239c0d0180f10656f
    Change ID: rlvkpnrzqnoowoytxnquwvuryrwnrmlp
    Author   : Test User <test.user@example.com> (2001-02-03 08:05:09)
    Committer: Test User <test.user@example.com> (2001-02-03 08:05:09)

        (no description set)

    file2            | 3 ++-
    {file1 => file3} | 0
    2 files changed, 2 insertions(+), 1 deletion(-)
    ");
}

#[test]
fn test_show_with_template() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    test_env
        .run_jj_in(&repo_path, ["new", "-m", "a new commit"])
        .success();

    let output = test_env.run_jj_in(&repo_path, ["show", "-T", "description"]);

    insta::assert_snapshot!(output, @"a new commit");
}

#[test]
fn test_show_with_no_template() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    let output = test_env.run_jj_in(&repo_path, ["show", "-T"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    error: a value is required for '--template <TEMPLATE>' but none was supplied

    For more information, try '--help'.
    Hint: The following template aliases are defined:
    - builtin_log_comfortable
    - builtin_log_compact
    - builtin_log_compact_full_description
    - builtin_log_detailed
    - builtin_log_node
    - builtin_log_node_ascii
    - builtin_log_oneline
    - builtin_op_log_comfortable
    - builtin_op_log_compact
    - builtin_op_log_node
    - builtin_op_log_node_ascii
    - builtin_op_log_oneline
    - commit_summary_separator
    - description_placeholder
    - email_placeholder
    - name_placeholder
    [exit status: 2]
    ");
}

#[test]
fn test_show_relative_timestamps() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    test_env.add_config(
        r#"
        [template-aliases]
        'format_timestamp(timestamp)' = 'timestamp.ago()'
        "#,
    );

    let output = test_env.run_jj_in(&repo_path, ["show"]);
    let timestamp_re = Regex::new(r"\([0-9]+ years ago\)").unwrap();
    let output = output.normalize_stdout_with(|s| {
        s.split_inclusive('\n')
            .skip(2)
            .map(|x| timestamp_re.replace_all(x, "(...timestamp...)"))
            .collect()
    });

    insta::assert_snapshot!(output, @r"
    Author   : Test User <test.user@example.com> (...timestamp...)
    Committer: Test User <test.user@example.com> (...timestamp...)

        (no description set)
    ");
}
