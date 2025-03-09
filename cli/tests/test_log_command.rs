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

use crate::common::to_toml_value;
use crate::common::TestEnvironment;

#[test]
fn test_log_with_empty_revision() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    let output = test_env.run_jj_in(&repo_path, ["log", "-r="]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    error: a value is required for '--revisions <REVSETS>' but none was supplied

    For more information, try '--help'.
    [exit status: 2]
    ");
}

#[test]
fn test_log_with_no_template() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    let output = test_env.run_jj_in(&repo_path, ["log", "-T"]);
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
fn test_log_with_or_without_diff() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "foo\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "add a file"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["new", "-m", "a new commit"])
        .success();
    std::fs::write(repo_path.join("file1"), "foo\nbar\n").unwrap();

    let output = test_env.run_jj_in(&repo_path, ["log", "-T", "description"]);
    insta::assert_snapshot!(output, @r"
    @  a new commit
    ○  add a file
    ◆
    ");

    let output = test_env.run_jj_in(&repo_path, ["log", "-T", "description", "-p"]);
    insta::assert_snapshot!(output, @r"
    @  a new commit
    │  Modified regular file file1:
    │     1    1: foo
    │          2: bar
    ○  add a file
    │  Added regular file file1:
    │          1: foo
    ◆
    ");

    let output = test_env.run_jj_in(&repo_path, ["log", "-T", "description", "--no-graph"]);
    insta::assert_snapshot!(output, @r"
    a new commit
    add a file
    ");

    // `-p` for default diff output, `-s` for summary
    let output = test_env.run_jj_in(&repo_path, ["log", "-T", "description", "-p", "-s"]);
    insta::assert_snapshot!(output, @r"
    @  a new commit
    │  M file1
    │  Modified regular file file1:
    │     1    1: foo
    │          2: bar
    ○  add a file
    │  A file1
    │  Added regular file file1:
    │          1: foo
    ◆
    ");

    // `-s` for summary, `--git` for git diff (which implies `-p`)
    let output = test_env.run_jj_in(&repo_path, ["log", "-T", "description", "-s", "--git"]);
    insta::assert_snapshot!(output, @r"
    @  a new commit
    │  M file1
    │  diff --git a/file1 b/file1
    │  index 257cc5642c..3bd1f0e297 100644
    │  --- a/file1
    │  +++ b/file1
    │  @@ -1,1 +1,2 @@
    │   foo
    │  +bar
    ○  add a file
    │  A file1
    │  diff --git a/file1 b/file1
    │  new file mode 100644
    │  index 0000000000..257cc5642c
    │  --- /dev/null
    │  +++ b/file1
    │  @@ -0,0 +1,1 @@
    │  +foo
    ◆
    ");

    // `-p` enables default "summary" output, so `-s` is noop
    let output = test_env.run_jj_in(
        &repo_path,
        [
            "log",
            "-T",
            "description",
            "-p",
            "-s",
            "--config=ui.diff.format=summary",
        ],
    );
    insta::assert_snapshot!(output, @r"
    @  a new commit
    │  M file1
    ○  add a file
    │  A file1
    ◆
    ");

    // `-p` enables default "color-words" diff output, so `--color-words` is noop
    let output = test_env.run_jj_in(
        &repo_path,
        ["log", "-T", "description", "-p", "--color-words"],
    );
    insta::assert_snapshot!(output, @r"
    @  a new commit
    │  Modified regular file file1:
    │     1    1: foo
    │          2: bar
    ○  add a file
    │  Added regular file file1:
    │          1: foo
    ◆
    ");

    // `--git` enables git diff, so `-p` is noop
    let output = test_env.run_jj_in(
        &repo_path,
        ["log", "-T", "description", "--no-graph", "-p", "--git"],
    );
    insta::assert_snapshot!(output, @r"
    a new commit
    diff --git a/file1 b/file1
    index 257cc5642c..3bd1f0e297 100644
    --- a/file1
    +++ b/file1
    @@ -1,1 +1,2 @@
     foo
    +bar
    add a file
    diff --git a/file1 b/file1
    new file mode 100644
    index 0000000000..257cc5642c
    --- /dev/null
    +++ b/file1
    @@ -0,0 +1,1 @@
    +foo
    ");

    // Cannot use both `--git` and `--color-words`
    let output = test_env.run_jj_in(
        &repo_path,
        [
            "log",
            "-T",
            "description",
            "--no-graph",
            "-p",
            "--git",
            "--color-words",
        ],
    );
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    error: the argument '--git' cannot be used with '--color-words'

    Usage: jj log --template <TEMPLATE> --no-graph --patch --git [FILESETS]...

    For more information, try '--help'.
    [exit status: 2]
    ");

    // `-s` with or without graph
    let output = test_env.run_jj_in(&repo_path, ["log", "-T", "description", "-s"]);
    insta::assert_snapshot!(output, @r"
    @  a new commit
    │  M file1
    ○  add a file
    │  A file1
    ◆
    ");
    let output = test_env.run_jj_in(&repo_path, ["log", "-T", "description", "--no-graph", "-s"]);
    insta::assert_snapshot!(output, @r"
    a new commit
    M file1
    add a file
    A file1
    ");

    // `--git` implies `-p`, with or without graph
    let output = test_env.run_jj_in(&repo_path, ["log", "-T", "description", "-r", "@", "--git"]);
    insta::assert_snapshot!(output, @r"
    @  a new commit
    │  diff --git a/file1 b/file1
    ~  index 257cc5642c..3bd1f0e297 100644
       --- a/file1
       +++ b/file1
       @@ -1,1 +1,2 @@
        foo
       +bar
    ");
    let output = test_env.run_jj_in(
        &repo_path,
        ["log", "-T", "description", "-r", "@", "--no-graph", "--git"],
    );
    insta::assert_snapshot!(output, @r"
    a new commit
    diff --git a/file1 b/file1
    index 257cc5642c..3bd1f0e297 100644
    --- a/file1
    +++ b/file1
    @@ -1,1 +1,2 @@
     foo
    +bar
    ");

    // `--color-words` implies `-p`, with or without graph
    let output = test_env.run_jj_in(
        &repo_path,
        ["log", "-T", "description", "-r", "@", "--color-words"],
    );
    insta::assert_snapshot!(output, @r"
    @  a new commit
    │  Modified regular file file1:
    ~     1    1: foo
               2: bar
    ");
    let output = test_env.run_jj_in(
        &repo_path,
        [
            "log",
            "-T",
            "description",
            "-r",
            "@",
            "--no-graph",
            "--color-words",
        ],
    );
    insta::assert_snapshot!(output, @r"
    a new commit
    Modified regular file file1:
       1    1: foo
            2: bar
    ");
}

#[test]
fn test_log_null_terminate_multiline_descriptions() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    test_env
        .run_jj_in(
            &repo_path,
            ["commit", "-m", "commit 1 line 1", "-m", "commit 1 line 2"],
        )
        .success();
    test_env
        .run_jj_in(
            &repo_path,
            ["commit", "-m", "commit 2 line 1", "-m", "commit 2 line 2"],
        )
        .success();
    test_env
        .run_jj_in(
            &repo_path,
            ["describe", "-m", "commit 3 line 1", "-m", "commit 3 line 2"],
        )
        .success();

    let output = test_env
        .run_jj_in(
            &repo_path,
            [
                "log",
                "-r",
                "~root()",
                "-T",
                r#"description ++ "\0""#,
                "--no-graph",
            ],
        )
        .success();
    insta::assert_debug_snapshot!(
        output.stdout.normalized(),
        @r#""commit 3 line 1\n\ncommit 3 line 2\n\0commit 2 line 1\n\ncommit 2 line 2\n\0commit 1 line 1\n\ncommit 1 line 2\n\0""#
    );
}

#[test]
fn test_log_shortest_accessors() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    let render = |rev, template| {
        test_env.run_jj_in(&repo_path, ["log", "--no-graph", "-r", rev, "-T", template])
    };
    test_env.add_config(
        r#"
        [template-aliases]
        'format_id(id)' = 'id.shortest(12).prefix() ++ "[" ++ id.shortest(12).rest() ++ "]"'
        "#,
    );

    std::fs::write(repo_path.join("file"), "original file\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "initial"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["bookmark", "c", "-r@", "original"])
        .success();
    insta::assert_snapshot!(
        render("original", r#"format_id(change_id) ++ " " ++ format_id(commit_id)"#),
        @"q[pvuntsmwlqt] e[0e22b9fae75][no newline]");

    // Create a chain of 10 commits
    for i in 1..10 {
        test_env
            .run_jj_in(&repo_path, ["new", "-m", &format!("commit{i}")])
            .success();
        std::fs::write(repo_path.join("file"), format!("file {i}\n")).unwrap();
    }
    // Create 2^3 duplicates of the chain
    for _ in 0..3 {
        test_env
            .run_jj_in(&repo_path, ["duplicate", "description(commit)"])
            .success();
    }

    insta::assert_snapshot!(
        render("original", r#"format_id(change_id) ++ " " ++ format_id(commit_id)"#),
        @"qpv[untsmwlqt] e0[e22b9fae75][no newline]");

    insta::assert_snapshot!(
        render("::@", r#"change_id.shortest() ++ " " ++ commit_id.shortest() ++ "\n""#), @r"
    wq ed
    km ef3
    kp af
    zn 23
    yo b87
    vr 1e
    yq 34
    ro cc
    mz 1b
    qpv e0
    zzz 00
    ");

    insta::assert_snapshot!(
        render("::@", r#"format_id(change_id) ++ " " ++ format_id(commit_id) ++ "\n""#), @r"
    wq[nwkozpkust] ed[e204633421]
    km[kuslswpqwq] ef3[d013266cd]
    kp[qxywonksrl] af[95b841712d]
    zn[kkpsqqskkl] 23[c1103d3427]
    yo[stqsxwqrlt] b87[aa9b24921]
    vr[uxwmqvtpmx] 1e[a31a205ce9]
    yq[osqzytrlsw] 34[befb94f4eb]
    ro[yxmykxtrkr] cc[0c127948ef]
    mz[vwutvlkqwt] 1b[7b715afc3f]
    qpv[untsmwlqt] e0[e22b9fae75]
    zzz[zzzzzzzzz] 00[0000000000]
    ");

    // Can get shorter prefixes in configured revset
    test_env.add_config(r#"revsets.short-prefixes = "(@----)::""#);
    insta::assert_snapshot!(
        render("::@", r#"format_id(change_id) ++ " " ++ format_id(commit_id) ++ "\n""#), @r"
    w[qnwkozpkust] ed[e204633421]
    km[kuslswpqwq] ef[3d013266cd]
    kp[qxywonksrl] a[f95b841712d]
    z[nkkpsqqskkl] 2[3c1103d3427]
    y[ostqsxwqrlt] b[87aa9b24921]
    vr[uxwmqvtpmx] 1e[a31a205ce9]
    yq[osqzytrlsw] 34[befb94f4eb]
    ro[yxmykxtrkr] cc[0c127948ef]
    mz[vwutvlkqwt] 1b[7b715afc3f]
    qpv[untsmwlqt] e0[e22b9fae75]
    zzz[zzzzzzzzz] 00[0000000000]
    ");

    // Can disable short prefixes by setting to empty string
    test_env.add_config(r#"revsets.short-prefixes = """#);
    insta::assert_snapshot!(
        render("::@", r#"format_id(change_id) ++ " " ++ format_id(commit_id) ++ "\n""#), @r"
    wq[nwkozpkust] ed[e204633421]
    km[kuslswpqwq] ef3[d013266cd]
    kp[qxywonksrl] af[95b841712d]
    zn[kkpsqqskkl] 23c[1103d3427]
    yo[stqsxwqrlt] b87[aa9b24921]
    vr[uxwmqvtpmx] 1e[a31a205ce9]
    yq[osqzytrlsw] 34[befb94f4eb]
    ro[yxmykxtrkr] cc[0c127948ef]
    mz[vwutvlkqwt] 1b[7b715afc3f]
    qpv[untsmwlqt] e0[e22b9fae75]
    zzz[zzzzzzzzz] 00[0000000000]
    ");
}

#[test]
fn test_log_bad_short_prefixes() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    // Suppress warning in the commit summary template
    test_env.add_config("template-aliases.'format_short_id(id)' = 'id.short(8)'");

    // Error on bad config of short prefixes
    test_env.add_config(r#"revsets.short-prefixes = "!nval!d""#);
    let output = test_env.run_jj_in(&repo_path, ["status"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Config error: Invalid `revsets.short-prefixes`
    Caused by:  --> 1:1
      |
    1 | !nval!d
      | ^---
      |
      = expected <strict_identifier> or <expression>
    For help, see https://jj-vcs.github.io/jj/latest/config/ or use `jj help -k config`.
    [exit status: 1]
    ");

    // Warn on resolution of short prefixes
    test_env.add_config("revsets.short-prefixes = 'missing'");
    let output = test_env.run_jj_in(&repo_path, ["log", "-Tcommit_id.shortest()"]);
    insta::assert_snapshot!(output, @r"
    @  2
    ◆  0
    ------- stderr -------
    Warning: In template expression
     --> 1:11
      |
    1 | commit_id.shortest()
      |           ^------^
      |
      = Failed to load short-prefixes index
    Failed to resolve short-prefixes disambiguation revset
    Revision `missing` doesn't exist
    ");

    // Error on resolution of short prefixes
    test_env.add_config("revsets.short-prefixes = 'missing'");
    let output = test_env.run_jj_in(&repo_path, ["log", "-r0"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Error: Failed to resolve short-prefixes disambiguation revset
    Caused by: Revision `missing` doesn't exist
    [exit status: 1]
    ");
}

#[test]
fn test_log_prefix_highlight_styled() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    fn prefix_format(len: Option<usize>) -> String {
        format!(
            r###"
            separate(" ",
              "Change",
              change_id.shortest({0}),
              description.first_line(),
              commit_id.shortest({0}),
              bookmarks,
            )
            "###,
            len.map(|l| l.to_string()).unwrap_or_default()
        )
    }

    std::fs::write(repo_path.join("file"), "original file\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "initial"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["bookmark", "c", "-r@", "original"])
        .success();
    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["log", "-r", "original", "-T", &prefix_format(Some(12))]), @r"
    @  Change qpvuntsmwlqt initial e0e22b9fae75 original
    │
    ~
    ");

    // Create a chain of 10 commits
    for i in 1..10 {
        test_env
            .run_jj_in(&repo_path, ["new", "-m", &format!("commit{i}")])
            .success();
        std::fs::write(repo_path.join("file"), format!("file {i}\n")).unwrap();
    }
    // Create 2^3 duplicates of the chain
    for _ in 0..3 {
        test_env
            .run_jj_in(&repo_path, ["duplicate", "description(commit)"])
            .success();
    }

    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["log", "-r", "original", "-T", &prefix_format(Some(12))]), @r"
    ○  Change qpvuntsmwlqt initial e0e22b9fae75 original
    │
    ~
    ");
    let output = test_env.run_jj_in(
        &repo_path,
        [
            "--color=always",
            "log",
            "-r",
            "@-----------..@",
            "-T",
            &prefix_format(Some(12)),
        ],
    );
    insta::assert_snapshot!(output, @"\u{1b}[1m\u{1b}[38;5;2m@\u{1b}[0m  Change \u{1b}[1m\u{1b}[38;5;5mwq\u{1b}[0m\u{1b}[38;5;8mnwkozpkust\u{1b}[39m commit9 \u{1b}[1m\u{1b}[38;5;4med\u{1b}[0m\u{1b}[38;5;8me204633421\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5mkm\u{1b}[0m\u{1b}[38;5;8mkuslswpqwq\u{1b}[39m commit8 \u{1b}[1m\u{1b}[38;5;4mef3\u{1b}[0m\u{1b}[38;5;8md013266cd\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5mkp\u{1b}[0m\u{1b}[38;5;8mqxywonksrl\u{1b}[39m commit7 \u{1b}[1m\u{1b}[38;5;4maf\u{1b}[0m\u{1b}[38;5;8m95b841712d\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5mzn\u{1b}[0m\u{1b}[38;5;8mkkpsqqskkl\u{1b}[39m commit6 \u{1b}[1m\u{1b}[38;5;4m23\u{1b}[0m\u{1b}[38;5;8mc1103d3427\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5myo\u{1b}[0m\u{1b}[38;5;8mstqsxwqrlt\u{1b}[39m commit5 \u{1b}[1m\u{1b}[38;5;4mb87\u{1b}[0m\u{1b}[38;5;8maa9b24921\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5mvr\u{1b}[0m\u{1b}[38;5;8muxwmqvtpmx\u{1b}[39m commit4 \u{1b}[1m\u{1b}[38;5;4m1e\u{1b}[0m\u{1b}[38;5;8ma31a205ce9\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5myq\u{1b}[0m\u{1b}[38;5;8mosqzytrlsw\u{1b}[39m commit3 \u{1b}[1m\u{1b}[38;5;4m34\u{1b}[0m\u{1b}[38;5;8mbefb94f4eb\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5mro\u{1b}[0m\u{1b}[38;5;8myxmykxtrkr\u{1b}[39m commit2 \u{1b}[1m\u{1b}[38;5;4mcc\u{1b}[0m\u{1b}[38;5;8m0c127948ef\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5mmz\u{1b}[0m\u{1b}[38;5;8mvwutvlkqwt\u{1b}[39m commit1 \u{1b}[1m\u{1b}[38;5;4m1b\u{1b}[0m\u{1b}[38;5;8m7b715afc3f\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5mqpv\u{1b}[0m\u{1b}[38;5;8muntsmwlqt\u{1b}[39m initial \u{1b}[1m\u{1b}[38;5;4me0\u{1b}[0m\u{1b}[38;5;8me22b9fae75\u{1b}[39m \u{1b}[38;5;5moriginal\u{1b}[39m\n\u{1b}[1m\u{1b}[38;5;14m◆\u{1b}[0m  Change \u{1b}[1m\u{1b}[38;5;5mzzz\u{1b}[0m\u{1b}[38;5;8mzzzzzzzzz\u{1b}[39m \u{1b}[1m\u{1b}[38;5;4m00\u{1b}[0m\u{1b}[38;5;8m0000000000\u{1b}[39m");
    let output = test_env.run_jj_in(
        &repo_path,
        [
            "--color=always",
            "log",
            "-r",
            "@-----------..@",
            "-T",
            &prefix_format(Some(3)),
        ],
    );
    insta::assert_snapshot!(output, @"\u{1b}[1m\u{1b}[38;5;2m@\u{1b}[0m  Change \u{1b}[1m\u{1b}[38;5;5mwq\u{1b}[0m\u{1b}[38;5;8mn\u{1b}[39m commit9 \u{1b}[1m\u{1b}[38;5;4med\u{1b}[0m\u{1b}[38;5;8me\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5mkm\u{1b}[0m\u{1b}[38;5;8mk\u{1b}[39m commit8 \u{1b}[1m\u{1b}[38;5;4mef3\u{1b}[0m\n○  Change \u{1b}[1m\u{1b}[38;5;5mkp\u{1b}[0m\u{1b}[38;5;8mq\u{1b}[39m commit7 \u{1b}[1m\u{1b}[38;5;4maf\u{1b}[0m\u{1b}[38;5;8m9\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5mzn\u{1b}[0m\u{1b}[38;5;8mk\u{1b}[39m commit6 \u{1b}[1m\u{1b}[38;5;4m23\u{1b}[0m\u{1b}[38;5;8mc\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5myo\u{1b}[0m\u{1b}[38;5;8ms\u{1b}[39m commit5 \u{1b}[1m\u{1b}[38;5;4mb87\u{1b}[0m\n○  Change \u{1b}[1m\u{1b}[38;5;5mvr\u{1b}[0m\u{1b}[38;5;8mu\u{1b}[39m commit4 \u{1b}[1m\u{1b}[38;5;4m1e\u{1b}[0m\u{1b}[38;5;8ma\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5myq\u{1b}[0m\u{1b}[38;5;8mo\u{1b}[39m commit3 \u{1b}[1m\u{1b}[38;5;4m34\u{1b}[0m\u{1b}[38;5;8mb\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5mro\u{1b}[0m\u{1b}[38;5;8my\u{1b}[39m commit2 \u{1b}[1m\u{1b}[38;5;4mcc\u{1b}[0m\u{1b}[38;5;8m0\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5mmz\u{1b}[0m\u{1b}[38;5;8mv\u{1b}[39m commit1 \u{1b}[1m\u{1b}[38;5;4m1b\u{1b}[0m\u{1b}[38;5;8m7\u{1b}[39m\n○  Change \u{1b}[1m\u{1b}[38;5;5mqpv\u{1b}[0m initial \u{1b}[1m\u{1b}[38;5;4me0\u{1b}[0m\u{1b}[38;5;8me\u{1b}[39m \u{1b}[38;5;5moriginal\u{1b}[39m\n\u{1b}[1m\u{1b}[38;5;14m◆\u{1b}[0m  Change \u{1b}[1m\u{1b}[38;5;5mzzz\u{1b}[0m \u{1b}[1m\u{1b}[38;5;4m00\u{1b}[0m\u{1b}[38;5;8m0\u{1b}[39m");
    let output = test_env.run_jj_in(
        &repo_path,
        [
            "--color=always",
            "log",
            "-r",
            "@-----------..@",
            "-T",
            &prefix_format(None),
        ],
    );
    insta::assert_snapshot!(output, @"\u{1b}[1m\u{1b}[38;5;2m@\u{1b}[0m  Change \u{1b}[1m\u{1b}[38;5;5mwq\u{1b}[0m commit9 \u{1b}[1m\u{1b}[38;5;4med\u{1b}[0m\n○  Change \u{1b}[1m\u{1b}[38;5;5mkm\u{1b}[0m commit8 \u{1b}[1m\u{1b}[38;5;4mef3\u{1b}[0m\n○  Change \u{1b}[1m\u{1b}[38;5;5mkp\u{1b}[0m commit7 \u{1b}[1m\u{1b}[38;5;4maf\u{1b}[0m\n○  Change \u{1b}[1m\u{1b}[38;5;5mzn\u{1b}[0m commit6 \u{1b}[1m\u{1b}[38;5;4m23\u{1b}[0m\n○  Change \u{1b}[1m\u{1b}[38;5;5myo\u{1b}[0m commit5 \u{1b}[1m\u{1b}[38;5;4mb87\u{1b}[0m\n○  Change \u{1b}[1m\u{1b}[38;5;5mvr\u{1b}[0m commit4 \u{1b}[1m\u{1b}[38;5;4m1e\u{1b}[0m\n○  Change \u{1b}[1m\u{1b}[38;5;5myq\u{1b}[0m commit3 \u{1b}[1m\u{1b}[38;5;4m34\u{1b}[0m\n○  Change \u{1b}[1m\u{1b}[38;5;5mro\u{1b}[0m commit2 \u{1b}[1m\u{1b}[38;5;4mcc\u{1b}[0m\n○  Change \u{1b}[1m\u{1b}[38;5;5mmz\u{1b}[0m commit1 \u{1b}[1m\u{1b}[38;5;4m1b\u{1b}[0m\n○  Change \u{1b}[1m\u{1b}[38;5;5mqpv\u{1b}[0m initial \u{1b}[1m\u{1b}[38;5;4me0\u{1b}[0m \u{1b}[38;5;5moriginal\u{1b}[39m\n\u{1b}[1m\u{1b}[38;5;14m◆\u{1b}[0m  Change \u{1b}[1m\u{1b}[38;5;5mzzz\u{1b}[0m \u{1b}[1m\u{1b}[38;5;4m00\u{1b}[0m");
}

#[test]
fn test_log_prefix_highlight_counts_hidden_commits() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    test_env.add_config(
        r#"
        [revsets]
        short-prefixes = "" # Disable short prefixes
        [template-aliases]
        'format_id(id)' = 'id.shortest(12).prefix() ++ "[" ++ id.shortest(12).rest() ++ "]"'
        "#,
    );

    let prefix_format = r#"
    separate(" ",
      "Change",
      format_id(change_id),
      description.first_line(),
      format_id(commit_id),
      bookmarks,
    )
    "#;

    std::fs::write(repo_path.join("file"), "original file\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "initial"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["bookmark", "c", "-r@", "original"])
        .success();
    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["log", "-r", "all()", "-T", prefix_format]), @r"
    @  Change q[pvuntsmwlqt] initial e0[e22b9fae75] original
    ◆  Change z[zzzzzzzzzzz] 0[00000000000]
    ");

    // Create 2^7 hidden commits
    test_env
        .run_jj_in(&repo_path, ["new", "root()", "-m", "extra"])
        .success();
    for _ in 0..7 {
        test_env
            .run_jj_in(&repo_path, ["duplicate", "description(extra)"])
            .success();
    }
    test_env
        .run_jj_in(&repo_path, ["abandon", "description(extra)"])
        .success();

    // The unique prefixes became longer.
    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["log", "-T", prefix_format]), @r"
    @  Change wq[nwkozpkust] 44[4c3c5066d3]
    │ ○  Change qpv[untsmwlqt] initial e0e[22b9fae75] original
    ├─╯
    ◆  Change zzz[zzzzzzzzz] 00[0000000000]
    ");
    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["log", "-r", "4", "-T", prefix_format]), @r"
    ------- stderr -------
    Error: Commit ID prefix `4` is ambiguous
    [exit status: 1]
    ");
    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["log", "-r", "44", "-T", prefix_format]), @r"
    @  Change wq[nwkozpkust] 44[4c3c5066d3]
    │
    ~
    ");
}

#[test]
fn test_log_short_shortest_length_parameter() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    let render = |template| test_env.run_jj_in(&repo_path, ["log", "-T", template]);

    insta::assert_snapshot!(
        render(r#"commit_id.short(0) ++ "|" ++ commit_id.shortest(0)"#), @r"
    @  |2
    ◆  |0
    ");
    insta::assert_snapshot!(
        render(r#"commit_id.short(-0) ++ "|" ++ commit_id.shortest(-0)"#), @r"
    @  |2
    ◆  |0
    ");
    insta::assert_snapshot!(
        render(r#"commit_id.short(-100) ++ "|" ++ commit_id.shortest(-100)"#), @r"
    @  <Error: out of range integral type conversion attempted>|<Error: out of range integral type conversion attempted>
    ◆  <Error: out of range integral type conversion attempted>|<Error: out of range integral type conversion attempted>
    ");
    insta::assert_snapshot!(
        render(r#"commit_id.short(100) ++ "|" ++ commit_id.shortest(100)"#), @r"
    @  230dd059e1b059aefc0da06a2e5a7dbf22362f22|230dd059e1b059aefc0da06a2e5a7dbf22362f22
    ◆  0000000000000000000000000000000000000000|0000000000000000000000000000000000000000
    ");
}

#[test]
fn test_log_author_format() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["log", "--revisions=@"]), @r"
    @  qpvuntsm test.user@example.com 2001-02-03 08:05:07 230dd059
    │  (empty) (no description set)
    ~
    ");

    let decl = "template-aliases.'format_short_signature(signature)'";
    insta::assert_snapshot!(
        test_env.run_jj_in(
            &repo_path,
            [
                "--config",
                &format!("{decl}='signature.email().local()'"),
                "log",
                "--revisions=@",
            ],
        ), @r"
    @  qpvuntsm test.user 2001-02-03 08:05:07 230dd059
    │  (empty) (no description set)
    ~
    ");
}

#[test]
fn test_log_divergence() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    let template = r#"description.first_line() ++ if(divergent, " !divergence!")"#;

    std::fs::write(repo_path.join("file"), "foo\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "description 1"])
        .success();
    // No divergence
    let output = test_env.run_jj_in(&repo_path, ["log", "-T", template]);
    insta::assert_snapshot!(output, @r"
    @  description 1
    ◆
    ");

    // Create divergence
    test_env
        .run_jj_in(
            &repo_path,
            ["describe", "-m", "description 2", "--at-operation", "@-"],
        )
        .success();
    let output = test_env.run_jj_in(&repo_path, ["log", "-T", template]);
    insta::assert_snapshot!(output, @r"
    @  description 1 !divergence!
    │ ○  description 2 !divergence!
    ├─╯
    ◆
    ------- stderr -------
    Concurrent modification detected, resolving automatically.
    ");
}

#[test]
fn test_log_reversed() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "first"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["new", "-m", "second"])
        .success();

    let output = test_env.run_jj_in(&repo_path, ["log", "-T", "description", "--reversed"]);
    insta::assert_snapshot!(output, @r"
    ◆
    ○  first
    @  second
    ");

    let output = test_env.run_jj_in(
        &repo_path,
        ["log", "-T", "description", "--reversed", "--no-graph"],
    );
    insta::assert_snapshot!(output, @r"
    first
    second
    ");
}

#[test]
fn test_log_filtered_by_path() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "foo\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "first"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["new", "-m", "second"])
        .success();
    std::fs::write(repo_path.join("file1"), "foo\nbar\n").unwrap();
    std::fs::write(repo_path.join("file2"), "baz\n").unwrap();

    let output = test_env.run_jj_in(&repo_path, ["log", "-T", "description", "file1"]);
    insta::assert_snapshot!(output, @r"
    @  second
    ○  first
    │
    ~
    ");

    let output = test_env.run_jj_in(&repo_path, ["log", "-T", "description", "file2"]);
    insta::assert_snapshot!(output, @r"
    @  second
    │
    ~
    ");

    let output = test_env.run_jj_in(&repo_path, ["log", "-T", "description", "-s", "file1"]);
    insta::assert_snapshot!(output, @r"
    @  second
    │  M file1
    ○  first
    │  A file1
    ~
    ");

    let output = test_env.run_jj_in(
        &repo_path,
        ["log", "-T", "description", "-s", "file2", "--no-graph"],
    );
    insta::assert_snapshot!(output, @r"
    second
    A file2
    ");

    // empty revisions are filtered out by "all()" fileset.
    let output = test_env.run_jj_in(&repo_path, ["log", "-Tdescription", "-s", "all()"]);
    insta::assert_snapshot!(output, @r"
    @  second
    │  M file1
    │  A file2
    ○  first
    │  A file1
    ~
    ");

    // "root:<path>" is resolved relative to the workspace root.
    let output = test_env.run_jj_in(
        ".",
        [
            "log",
            "-R",
            repo_path.to_str().unwrap(),
            "-Tdescription",
            "-s",
            "root:file1",
        ],
    );
    insta::assert_snapshot!(output.normalize_backslash(), @r"
    @  second
    │  M repo/file1
    ○  first
    │  A repo/file1
    ~
    ");

    // files() revset doesn't filter the diff.
    let output = test_env.run_jj_in(
        &repo_path,
        [
            "log",
            "-T",
            "description",
            "-s",
            "-rfiles(file2)",
            "--no-graph",
        ],
    );
    insta::assert_snapshot!(output, @r"
    second
    M file1
    A file2
    ");
}

#[test]
fn test_log_limit() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "a"])
        .success();
    std::fs::write(repo_path.join("a"), "").unwrap();
    test_env.run_jj_in(&repo_path, ["new", "-m", "b"]).success();
    std::fs::write(repo_path.join("b"), "").unwrap();
    test_env
        .run_jj_in(&repo_path, ["new", "-m", "c", "description(a)"])
        .success();
    std::fs::write(repo_path.join("c"), "").unwrap();
    test_env
        .run_jj_in(
            &repo_path,
            ["new", "-m", "d", "description(c)", "description(b)"],
        )
        .success();

    let output = test_env.run_jj_in(&repo_path, ["log", "-T", "description", "--limit=3"]);
    insta::assert_snapshot!(output, @r"
    @    d
    ├─╮
    │ ○  b
    ○ │  c
    ├─╯
    ");

    // Applied on sorted DAG
    let output = test_env.run_jj_in(&repo_path, ["log", "-T", "description", "--limit=2"]);
    insta::assert_snapshot!(output, @r"
    @    d
    ├─╮
    │ ○  b
    ");

    let output = test_env.run_jj_in(
        &repo_path,
        ["log", "-T", "description", "--limit=2", "--no-graph"],
    );
    insta::assert_snapshot!(output, @r"
    d
    c
    ");

    // Applied on reversed DAG: Because the node "a" is omitted, "b" and "c" are
    // rendered as roots.
    let output = test_env.run_jj_in(
        &repo_path,
        ["log", "-T", "description", "--limit=3", "--reversed"],
    );
    insta::assert_snapshot!(output, @r"
    ○  c
    │ ○  b
    ├─╯
    @  d
    ");
    let output = test_env.run_jj_in(
        &repo_path,
        [
            "log",
            "-T",
            "description",
            "--limit=3",
            "--reversed",
            "--no-graph",
        ],
    );
    insta::assert_snapshot!(output, @r"
    b
    c
    d
    ");

    // Applied on filtered commits
    let output = test_env.run_jj_in(
        &repo_path,
        ["log", "-T", "description", "--limit=1", "b", "c"],
    );
    insta::assert_snapshot!(output, @r"
    ○  c
    │
    ~
    ");
}

#[test]
fn test_log_warn_path_might_be_revset() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "foo\n").unwrap();

    // Don't warn if the file actually exists.
    let output = test_env.run_jj_in(&repo_path, ["log", "file1", "-T", "description"]);
    insta::assert_snapshot!(output, @r"
    @
    │
    ~
    ");

    // Warn for `jj log .` specifically, for former Mercurial users.
    let output = test_env.run_jj_in(&repo_path, ["log", ".", "-T", "description"]);
    insta::assert_snapshot!(output, @r#"
    @
    │
    ~
    ------- stderr -------
    Warning: The argument "." is being interpreted as a fileset expression, but this is often not useful because all non-empty commits touch '.'. If you meant to show the working copy commit, pass -r '@' instead.
    "#);

    // ...but checking `jj log .` makes sense in a subdirectory.
    let subdir = repo_path.join("dir");
    std::fs::create_dir_all(&subdir).unwrap();
    let output = test_env.run_jj_in(&subdir, ["log", "."]);
    insta::assert_snapshot!(output, @"");

    // Warn for `jj log @` instead of `jj log -r @`.
    let output = test_env.run_jj_in(&repo_path, ["log", "@", "-T", "description"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Warning: The argument "@" is being interpreted as a fileset expression. To specify a revset, pass -r "@" instead.
    "#);

    // Warn when there's no path with the provided name.
    let output = test_env.run_jj_in(&repo_path, ["log", "file2", "-T", "description"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Warning: The argument "file2" is being interpreted as a fileset expression. To specify a revset, pass -r "file2" instead.
    "#);

    // If an explicit revision is provided, then suppress the warning.
    let output = test_env.run_jj_in(&repo_path, ["log", "@", "-r", "@", "-T", "description"]);
    insta::assert_snapshot!(output, @"");
}

#[test]
fn test_default_revset() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "foo\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "add a file"])
        .success();

    // Set configuration to only show the root commit.
    test_env.add_config(r#"revsets.log = "root()""#);

    // Log should only contain one line (for the root commit), and not show the
    // commit created above.
    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["log", "-T", "commit_id"]), @"◆  0000000000000000000000000000000000000000");

    // The default revset is not used if a path is specified
    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["log", "file1", "-T", "description"]), @r"
    @  add a file
    │
    ~
    ");
}

#[test]
fn test_default_revset_per_repo() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    std::fs::write(repo_path.join("file1"), "foo\n").unwrap();
    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "add a file"])
        .success();

    // Set configuration to only show the root commit.
    std::fs::write(
        repo_path.join(".jj/repo/config.toml"),
        r#"revsets.log = "root()""#,
    )
    .unwrap();

    // Log should only contain one line (for the root commit), and not show the
    // commit created above.
    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["log", "-T", "commit_id"]), @"◆  0000000000000000000000000000000000000000");
}

#[test]
fn test_multiple_revsets() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    for name in ["foo", "bar", "baz"] {
        test_env
            .run_jj_in(&repo_path, ["new", "-m", name])
            .success();
        test_env
            .run_jj_in(&repo_path, ["bookmark", "create", "-r@", name])
            .success();
    }

    // Default revset should be overridden if one or more -r options are specified.
    test_env.add_config(r#"revsets.log = "root()""#);

    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["log", "-T", "bookmarks", "-rfoo"]), @r"
    ○  foo
    │
    ~
    ");
    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["log", "-T", "bookmarks", "-rfoo", "-rbar", "-rbaz"]), @r"
    @  baz
    ○  bar
    ○  foo
    │
    ~
    ");
    insta::assert_snapshot!(
        test_env.run_jj_in(&repo_path, ["log", "-T", "bookmarks", "-rfoo", "-rfoo"]), @r"
    ○  foo
    │
    ~
    ");
}

#[test]
fn test_graph_template_color() {
    // Test that color codes from a multi-line template don't span the graph lines.
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    test_env
        .run_jj_in(
            &repo_path,
            ["describe", "-m", "first line\nsecond line\nthird line"],
        )
        .success();
    test_env
        .run_jj_in(&repo_path, ["new", "-m", "single line"])
        .success();

    test_env.add_config(
        r#"[colors]
        description = "red"
        "working_copy description" = "green"
        "#,
    );

    // First test without color for comparison
    let template = r#"label(if(current_working_copy, "working_copy"), description)"#;
    let output = test_env.run_jj_in(&repo_path, ["log", "-T", template]);
    insta::assert_snapshot!(output, @r"
    @  single line
    ○  first line
    │  second line
    │  third line
    ◆
    ");
    let output = test_env.run_jj_in(&repo_path, ["--color=always", "log", "-T", template]);
    insta::assert_snapshot!(output, @"\u{1b}[1m\u{1b}[38;5;2m@\u{1b}[0m  \u{1b}[1m\u{1b}[38;5;2msingle line\u{1b}[0m\n○  \u{1b}[38;5;1mfirst line\u{1b}[39m\n│  \u{1b}[38;5;1msecond line\u{1b}[39m\n│  \u{1b}[38;5;1mthird line\u{1b}[39m\n\u{1b}[1m\u{1b}[38;5;14m◆\u{1b}[0m");
    let output = test_env.run_jj_in(&repo_path, ["--color=debug", "log", "-T", template]);
    insta::assert_snapshot!(output, @"\u{1b}[1m\u{1b}[38;5;2m<<node working_copy::@>>\u{1b}[0m  \u{1b}[1m\u{1b}[38;5;2m<<log working_copy description::single line>>\u{1b}[0m\n<<node::○>>  \u{1b}[38;5;1m<<log description::first line>>\u{1b}[39m\n│  \u{1b}[38;5;1m<<log description::second line>>\u{1b}[39m\n│  \u{1b}[38;5;1m<<log description::third line>>\u{1b}[39m\n\u{1b}[1m\u{1b}[38;5;14m<<node immutable::◆>>\u{1b}[0m");
}

#[test]
fn test_graph_styles() {
    // Test that different graph styles are available.
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    test_env
        .run_jj_in(&repo_path, ["commit", "-m", "initial"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["commit", "-m", "main bookmark 1"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "main bookmark 2"])
        .success();
    test_env
        .run_jj_in(
            &repo_path,
            ["new", "-m", "side bookmark\nwith\nlong\ndescription"],
        )
        .success();
    test_env
        .run_jj_in(
            &repo_path,
            [
                "new",
                "-m",
                "merge",
                r#"description("main bookmark 1")"#,
                "@",
            ],
        )
        .success();

    // Default (curved) style
    let output = test_env.run_jj_in(&repo_path, ["log", "-T=description"]);
    insta::assert_snapshot!(output, @r"
    @    merge
    ├─╮
    │ ○  side bookmark
    │ │  with
    │ │  long
    │ │  description
    │ ○  main bookmark 2
    ├─╯
    ○  main bookmark 1
    ○  initial
    ◆
    ");

    // ASCII style
    test_env.add_config(r#"ui.graph.style = "ascii""#);
    let output = test_env.run_jj_in(&repo_path, ["log", "-T=description"]);
    insta::assert_snapshot!(output, @r"
    @    merge
    |\
    | o  side bookmark
    | |  with
    | |  long
    | |  description
    | o  main bookmark 2
    |/
    o  main bookmark 1
    o  initial
    +
    ");

    // Large ASCII style
    test_env.add_config(r#"ui.graph.style = "ascii-large""#);
    let output = test_env.run_jj_in(&repo_path, ["log", "-T=description"]);
    insta::assert_snapshot!(output, @r"
    @     merge
    |\
    | \
    |  o  side bookmark
    |  |  with
    |  |  long
    |  |  description
    |  o  main bookmark 2
    | /
    |/
    o  main bookmark 1
    o  initial
    +
    ");

    // Curved style
    test_env.add_config(r#"ui.graph.style = "curved""#);
    let output = test_env.run_jj_in(&repo_path, ["log", "-T=description"]);
    insta::assert_snapshot!(output, @r"
    @    merge
    ├─╮
    │ ○  side bookmark
    │ │  with
    │ │  long
    │ │  description
    │ ○  main bookmark 2
    ├─╯
    ○  main bookmark 1
    ○  initial
    ◆
    ");

    // Square style
    test_env.add_config(r#"ui.graph.style = "square""#);
    let output = test_env.run_jj_in(&repo_path, ["log", "-T=description"]);
    insta::assert_snapshot!(output, @r"
    @    merge
    ├─┐
    │ ○  side bookmark
    │ │  with
    │ │  long
    │ │  description
    │ ○  main bookmark 2
    ├─┘
    ○  main bookmark 1
    ○  initial
    ◆
    ");

    // Invalid style name
    let output = test_env.run_jj_in(&repo_path, ["log", "--config=ui.graph.style=unknown"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Config error: Invalid type or value for ui.graph.style
    Caused by: unknown variant `unknown`, expected one of `ascii`, `ascii-large`, `curved`, `square`

    For help, see https://jj-vcs.github.io/jj/latest/config/ or use `jj help -k config`.
    [exit status: 1]
    ");
}

#[test]
fn test_log_word_wrap() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    let render = |args: &[&str], columns: u32, word_wrap: bool| {
        let word_wrap = to_toml_value(word_wrap);
        test_env.run_jj_with(|cmd| {
            cmd.current_dir(&repo_path)
                .args(args)
                .arg(format!("--config=ui.log-word-wrap={word_wrap}"))
                .env("COLUMNS", columns.to_string())
        })
    };

    test_env
        .run_jj_in(&repo_path, ["commit", "-m", "main bookmark 1"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "main bookmark 2"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["new", "-m", "side"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["new", "-m", "merge", "@--", "@"])
        .success();

    // ui.log-word-wrap option applies to both graph/no-graph outputs
    insta::assert_snapshot!(render(&["log", "-r@"], 40, false), @r"
    @  mzvwutvl test.user@example.com 2001-02-03 08:05:11 f3efbd00
    │  (empty) merge
    ~
    ");
    insta::assert_snapshot!(render(&["log", "-r@"], 40, true), @r"
    @  mzvwutvl test.user@example.com
    │  2001-02-03 08:05:11 f3efbd00
    ~  (empty) merge
    ");
    insta::assert_snapshot!(render(&["log", "--no-graph", "-r@"], 40, false), @r"
    mzvwutvl test.user@example.com 2001-02-03 08:05:11 f3efbd00
    (empty) merge
    ");
    insta::assert_snapshot!(render(&["log", "--no-graph", "-r@"], 40, true), @r"
    mzvwutvl test.user@example.com
    2001-02-03 08:05:11 f3efbd00
    (empty) merge
    ");

    // Color labels should be preserved
    insta::assert_snapshot!(render(&["log", "-r@", "--color=always"], 40, true), @"\u{1b}[1m\u{1b}[38;5;2m@\u{1b}[0m  \u{1b}[1m\u{1b}[38;5;13mm\u{1b}[38;5;8mzvwutvl\u{1b}[39m \u{1b}[38;5;3mtest.user@example.com\u{1b}[39m\u{1b}[0m\n│  \u{1b}[1m\u{1b}[38;5;14m2001-02-03 08:05:11\u{1b}[39m \u{1b}[38;5;12mf\u{1b}[38;5;8m3efbd00\u{1b}[39m\u{1b}[0m\n~  \u{1b}[1m\u{1b}[38;5;10m(empty)\u{1b}[39m merge\u{1b}[0m");

    // Graph width should be subtracted from the term width
    let template = r#""0 1 2 3 4 5 6 7 8 9""#;
    insta::assert_snapshot!(render(&["log", "-T", template], 10, true), @r"
    @    0 1 2
    ├─╮  3 4 5
    │ │  6 7 8
    │ │  9
    │ ○  0 1 2
    │ │  3 4 5
    │ │  6 7 8
    │ │  9
    │ ○  0 1 2
    ├─╯  3 4 5
    │    6 7 8
    │    9
    ○  0 1 2 3
    │  4 5 6 7
    │  8 9
    ◆  0 1 2 3
       4 5 6 7
       8 9
    ");

    // Shouldn't panic with $COLUMNS < graph_width
    insta::assert_snapshot!(render(&["log", "-r@"], 0, true), @r"
    @  mzvwutvl
    │  test.user@example.com
    ~  2001-02-03
       08:05:11
       f3efbd00
       (empty)
       merge
    ");
    insta::assert_snapshot!(render(&["log", "-r@"], 1, true), @r"
    @  mzvwutvl
    │  test.user@example.com
    ~  2001-02-03
       08:05:11
       f3efbd00
       (empty)
       merge
    ");
}

#[test]
fn test_log_diff_stat_width() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");
    let render = |args: &[&str], columns: u32| {
        test_env.run_jj_with(|cmd| {
            cmd.current_dir(&repo_path)
                .args(args)
                .env("COLUMNS", columns.to_string())
        })
    };

    std::fs::write(repo_path.join("file1"), "foo\n".repeat(100)).unwrap();
    test_env.run_jj_in(&repo_path, ["new", "root()"]).success();
    std::fs::write(repo_path.join("file2"), "foo\n".repeat(100)).unwrap();

    insta::assert_snapshot!(render(&["log", "--stat", "--no-graph"], 30), @r"
    rlvkpnrz test.user@example.com 2001-02-03 08:05:09 287520bf
    (no description set)
    file2 | 100 +++++++++++++++
    1 file changed, 100 insertions(+), 0 deletions(-)
    qpvuntsm test.user@example.com 2001-02-03 08:05:08 e292def1
    (no description set)
    file1 | 100 +++++++++++++++
    1 file changed, 100 insertions(+), 0 deletions(-)
    zzzzzzzz root() 00000000
    0 files changed, 0 insertions(+), 0 deletions(-)
    ");

    // Graph width should be subtracted
    insta::assert_snapshot!(render(&["log", "--stat"], 30), @r"
    @  rlvkpnrz test.user@example.com 2001-02-03 08:05:09 287520bf
    │  (no description set)
    │  file2 | 100 ++++++++++++
    │  1 file changed, 100 insertions(+), 0 deletions(-)
    │ ○  qpvuntsm test.user@example.com 2001-02-03 08:05:08 e292def1
    ├─╯  (no description set)
    │    file1 | 100 ++++++++++
    │    1 file changed, 100 insertions(+), 0 deletions(-)
    ◆  zzzzzzzz root() 00000000
       0 files changed, 0 insertions(+), 0 deletions(-)
    ");
}

#[test]
fn test_elided() {
    // Test that elided commits are shown as synthetic nodes.
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "initial"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["new", "-m", "main bookmark 1"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["new", "-m", "main bookmark 2"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["new", "@--", "-m", "side bookmark 1"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["new", "-m", "side bookmark 2"])
        .success();
    test_env
        .run_jj_in(
            &repo_path,
            [
                "new",
                "-m",
                "merge",
                r#"description("main bookmark 2")"#,
                "@",
            ],
        )
        .success();

    let get_log = |revs: &str| {
        test_env.run_jj_in(
            &repo_path,
            ["log", "-T", r#"description ++ "\n""#, "-r", revs],
        )
    };

    // Test the setup
    insta::assert_snapshot!(get_log("::"), @r"
    @    merge
    ├─╮
    │ ○  side bookmark 2
    │ │
    │ ○  side bookmark 1
    │ │
    ○ │  main bookmark 2
    │ │
    ○ │  main bookmark 1
    ├─╯
    ○  initial
    │
    ◆
    ");

    // Elide some commits from each side of the merge. It's unclear that a revision
    // was skipped on the left side.
    test_env.add_config("ui.log-synthetic-elided-nodes = false");
    insta::assert_snapshot!(get_log("@ | @- | description(initial)"), @r"
    @    merge
    ├─╮
    │ ○  side bookmark 2
    │ ╷
    ○ ╷  main bookmark 2
    ├─╯
    ○  initial
    │
    ~
    ");

    // Elide shared commits. It's unclear that a revision was skipped on the right
    // side (#1252).
    insta::assert_snapshot!(get_log("@-- | root()"), @r"
    ○  side bookmark 1
    ╷
    ╷ ○  main bookmark 1
    ╭─╯
    ◆
    ");

    // Now test the same thing with synthetic nodes for elided commits

    // Elide some commits from each side of the merge
    test_env.add_config("ui.log-synthetic-elided-nodes = true");
    insta::assert_snapshot!(get_log("@ | @- | description(initial)"), @r"
    @    merge
    ├─╮
    │ ○  side bookmark 2
    │ │
    │ ~  (elided revisions)
    ○ │  main bookmark 2
    │ │
    ~ │  (elided revisions)
    ├─╯
    ○  initial
    │
    ~
    ");

    // Elide shared commits. To keep the implementation simple, it still gets
    // rendered as two synthetic nodes.
    insta::assert_snapshot!(get_log("@-- | root()"), @r"
    ○  side bookmark 1
    │
    ~  (elided revisions)
    │ ○  main bookmark 1
    │ │
    │ ~  (elided revisions)
    ├─╯
    ◆
    ");
}

#[test]
fn test_log_with_custom_symbols() {
    // Test that elided commits are shown as synthetic nodes.
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    test_env
        .run_jj_in(&repo_path, ["describe", "-m", "initial"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["new", "-m", "main bookmark 1"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["new", "-m", "main bookmark 2"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["new", "@--", "-m", "side bookmark 1"])
        .success();
    test_env
        .run_jj_in(&repo_path, ["new", "-m", "side bookmark 2"])
        .success();
    test_env
        .run_jj_in(
            &repo_path,
            [
                "new",
                "-m",
                "merge",
                r#"description("main bookmark 2")"#,
                "@",
            ],
        )
        .success();

    let get_log = |revs: &str| {
        test_env.run_jj_in(
            &repo_path,
            ["log", "-T", r#"description ++ "\n""#, "-r", revs],
        )
    };

    // Simple test with showing default and elided nodes.
    test_env.add_config(
        r###"
        ui.log-synthetic-elided-nodes = true
        templates.log_node = 'if(self, if(current_working_copy, "$", if(root, "┴", "┝")), "🮀")'
        "###,
    );
    insta::assert_snapshot!(get_log("@ | @- | description(initial) | root()"), @r"
    $    merge
    ├─╮
    │ ┝  side bookmark 2
    │ │
    │ 🮀  (elided revisions)
    ┝ │  main bookmark 2
    │ │
    🮀 │  (elided revisions)
    ├─╯
    ┝  initial
    │
    ┴
    ");

    // Simple test with showing default and elided nodes, ascii style.
    test_env.add_config(
        r###"
        ui.log-synthetic-elided-nodes = true
        ui.graph.style = 'ascii'
        templates.log_node = 'if(self, if(current_working_copy, "$", if(root, "^", "*")), ":")'
        "###,
    );
    insta::assert_snapshot!(get_log("@ | @- | description(initial) | root()"), @r"
    $    merge
    |\
    | *  side bookmark 2
    | |
    | :  (elided revisions)
    * |  main bookmark 2
    | |
    : |  (elided revisions)
    |/
    *  initial
    |
    ^
    ");
}

#[test]
fn test_log_full_description_template() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let repo_path = test_env.env_root().join("repo");

    test_env
        .run_jj_in(
            &repo_path,
            [
                "describe",
                "-m",
                "this is commit with a multiline description\n\n<full description>",
            ],
        )
        .success();

    let output = test_env.run_jj_in(
        &repo_path,
        ["log", "-T", "builtin_log_compact_full_description"],
    );
    insta::assert_snapshot!(output, @r"
    @  qpvuntsm test.user@example.com 2001-02-03 08:05:08 1c504ec6
    │  (empty) this is commit with a multiline description
    │
    │  <full description>
    │
    ◆  zzzzzzzz root() 00000000
    ");
}
