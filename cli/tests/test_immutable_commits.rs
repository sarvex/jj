// Copyright 2023 The Jujutsu Authors
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

use crate::common::TestEnvironment;

#[test]
fn test_rewrite_immutable_generic() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");
    work_dir.write_file("file", "a");
    work_dir.run_jj(["describe", "-m=a"]).success();
    work_dir.run_jj(["new", "-m=b"]).success();
    work_dir.write_file("file", "b");
    work_dir
        .run_jj(["bookmark", "create", "-r@", "main"])
        .success();
    work_dir.run_jj(["new", "main-", "-m=c"]).success();
    work_dir.write_file("file", "c");
    let output = work_dir.run_jj(["log"]);
    insta::assert_snapshot!(output, @r"
    @  mzvwutvl test.user@example.com 2001-02-03 08:05:12 7adb43e8
    │  c
    │ ○  kkmpptxz test.user@example.com 2001-02-03 08:05:10 main 72e1b68c
    ├─╯  b
    ○  qpvuntsm test.user@example.com 2001-02-03 08:05:08 b84b821b
    │  a
    ◆  zzzzzzzz root() 00000000
    [EOF]
    ");

    // Cannot rewrite a commit in the configured set
    test_env.add_config(r#"revset-aliases."immutable_heads()" = "main""#);
    let output = work_dir.run_jj(["edit", "main"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit 72e1b68cbcf2 is immutable
    Hint: Could not modify commit: kkmpptxz 72e1b68c main | b
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // Cannot rewrite an ancestor of the configured set
    let output = work_dir.run_jj(["edit", "main-"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit b84b821b8a2b is immutable
    Hint: Could not modify commit: qpvuntsm b84b821b a
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 2 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // Cannot rewrite the root commit even with an empty set of immutable commits
    test_env.add_config(r#"revset-aliases."immutable_heads()" = "none()""#);
    let output = work_dir.run_jj(["edit", "root()"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Error: The root commit 000000000000 is immutable
    [EOF]
    [exit status: 1]
    ");

    // Error mutating the repo if immutable_heads() uses a ref that can't be
    // resolved
    test_env.add_config(r#"revset-aliases."immutable_heads()" = "bookmark_that_does_not_exist""#);
    // Suppress warning in the commit summary template
    test_env.add_config("template-aliases.'format_short_id(id)' = 'id.short(8)'");
    let output = work_dir.run_jj(["new", "main"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Config error: Invalid `revset-aliases.immutable_heads()`
    Caused by: Revision `bookmark_that_does_not_exist` doesn't exist
    For help, see https://jj-vcs.github.io/jj/latest/config/ or use `jj help -k config`.
    [EOF]
    [exit status: 1]
    ");

    // Can use --ignore-immutable to override
    test_env.add_config(r#"revset-aliases."immutable_heads()" = "main""#);
    let output = work_dir.run_jj(["--ignore-immutable", "edit", "main"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Working copy  (@) now at: kkmpptxz 72e1b68c main | b
    Parent commit (@-)      : qpvuntsm b84b821b a
    Added 0 files, modified 1 files, removed 0 files
    [EOF]
    ");
    // ... but not the root commit
    let output = work_dir.run_jj(["--ignore-immutable", "edit", "root()"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Error: The root commit 000000000000 is immutable
    [EOF]
    [exit status: 1]
    ");

    // Mutating the repo works if ref is wrapped in present()
    test_env.add_config(
        r#"revset-aliases."immutable_heads()" = "present(bookmark_that_does_not_exist)""#,
    );
    let output = work_dir.run_jj(["new", "main"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Working copy  (@) now at: wqnwkozp fc921593 (empty) (no description set)
    Parent commit (@-)      : kkmpptxz 72e1b68c main | b
    [EOF]
    ");

    // immutable_heads() of different arity doesn't shadow the 0-ary one
    test_env.add_config(r#"revset-aliases."immutable_heads(foo)" = "none()""#);
    let output = work_dir.run_jj(["edit", "root()"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Error: The root commit 000000000000 is immutable
    [EOF]
    [exit status: 1]
    ");
}

#[test]
fn test_new_wc_commit_when_wc_immutable() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init"]).success();
    let work_dir = test_env.work_dir("");
    work_dir
        .run_jj(["bookmark", "create", "-r@", "main"])
        .success();
    test_env.add_config(r#"revset-aliases."immutable_heads()" = "main""#);
    work_dir.run_jj(["new", "-m=a"]).success();
    let output = work_dir.run_jj(["bookmark", "set", "main", "-r@"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Moved 1 bookmarks to kkmpptxz 7ec46fca main | (empty) a
    Warning: The working-copy commit in workspace 'default' became immutable, so a new commit has been created on top of it.
    Working copy  (@) now at: zsuskuln 2ec95daa (empty) (no description set)
    Parent commit (@-)      : kkmpptxz 7ec46fca main | (empty) a
    [EOF]
    ");
}

#[test]
fn test_immutable_heads_set_to_working_copy() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init"]).success();
    let work_dir = test_env.work_dir("");
    work_dir
        .run_jj(["bookmark", "create", "-r@", "main"])
        .success();
    test_env.add_config(r#"revset-aliases."immutable_heads()" = "@""#);
    let output = work_dir.run_jj(["new", "-m=a"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Warning: The working-copy commit in workspace 'default' became immutable, so a new commit has been created on top of it.
    Working copy  (@) now at: pmmvwywv 12bba747 (empty) (no description set)
    Parent commit (@-)      : kkmpptxz b0930409 (empty) a
    [EOF]
    ");
}

#[test]
fn test_new_wc_commit_when_wc_immutable_multi_workspace() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");
    work_dir
        .run_jj(["bookmark", "create", "-r@", "main"])
        .success();
    test_env.add_config(r#"revset-aliases."immutable_heads()" = "main""#);
    work_dir.run_jj(["new", "-m=a"]).success();
    work_dir
        .run_jj(["workspace", "add", "../workspace1"])
        .success();
    let workspace1_dir = test_env.work_dir("workspace1");
    workspace1_dir.run_jj(["edit", "default@"]).success();
    let output = work_dir.run_jj(["bookmark", "set", "main", "-r@"]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Moved 1 bookmarks to kkmpptxz 7796c4df main | (empty) a
    Warning: The working-copy commit in workspace 'default' became immutable, so a new commit has been created on top of it.
    Warning: The working-copy commit in workspace 'workspace1' became immutable, so a new commit has been created on top of it.
    Working copy  (@) now at: royxmykx 896465c4 (empty) (no description set)
    Parent commit (@-)      : kkmpptxz 7796c4df main | (empty) a
    [EOF]
    ");
    workspace1_dir
        .run_jj(["workspace", "update-stale"])
        .success();
    let output = workspace1_dir.run_jj(["log", "--no-graph"]);
    insta::assert_snapshot!(output, @r"
    nppvrztz test.user@example.com 2001-02-03 08:05:11 workspace1@ ee0671fd
    (empty) (no description set)
    royxmykx test.user@example.com 2001-02-03 08:05:12 default@ 896465c4
    (empty) (no description set)
    kkmpptxz test.user@example.com 2001-02-03 08:05:09 main 7796c4df
    (empty) a
    zzzzzzzz root() 00000000
    [EOF]
    ");
}

#[test]
fn test_rewrite_immutable_commands() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");
    work_dir.write_file("file", "a");
    work_dir.run_jj(["describe", "-m=a"]).success();
    work_dir.run_jj(["new", "-m=b"]).success();
    work_dir.write_file("file", "b");
    work_dir.run_jj(["new", "@-", "-m=c"]).success();
    work_dir.write_file("file", "c");
    work_dir
        .run_jj(["new", "all:visible_heads()", "-m=merge"])
        .success();
    // Create another file to make sure the merge commit isn't empty (to satisfy `jj
    // split`) and still has a conflict (to satisfy `jj resolve`).
    work_dir.write_file("file2", "merged");
    work_dir
        .run_jj(["bookmark", "create", "-r@", "main"])
        .success();
    work_dir.run_jj(["new", "description(b)"]).success();
    work_dir.write_file("file", "w");
    test_env.add_config(r#"revset-aliases."immutable_heads()" = "main""#);
    test_env.add_config(r#"revset-aliases."trunk()" = "main""#);

    // Log shows mutable commits, their parents, and trunk() by default
    let output = work_dir.run_jj(["log"]);
    insta::assert_snapshot!(output, @r"
    @  yqosqzyt test.user@example.com 2001-02-03 08:05:14 55641cc5
    │  (no description set)
    │ ◆  mzvwutvl test.user@example.com 2001-02-03 08:05:12 main bcab555f conflict
    ╭─┤  merge
    │ │
    │ ~
    │
    ◆  kkmpptxz test.user@example.com 2001-02-03 08:05:10 72e1b68c
    │  b
    ~
    [EOF]
    ");

    // abandon
    let output = work_dir.run_jj(["abandon", "main"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // absorb
    let output = work_dir.run_jj(["absorb", "--into=::@-"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit 72e1b68cbcf2 is immutable
    Hint: Could not modify commit: kkmpptxz 72e1b68c b
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 2 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // chmod
    let output = work_dir.run_jj(["file", "chmod", "-r=main", "x", "file"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // describe
    let output = work_dir.run_jj(["describe", "main"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // diffedit
    let output = work_dir.run_jj(["diffedit", "-r=main"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // edit
    let output = work_dir.run_jj(["edit", "main"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // new --insert-before
    let output = work_dir.run_jj(["new", "--insert-before", "main"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // new --insert-after parent_of_main
    let output = work_dir.run_jj(["new", "--insert-after", "description(b)"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // parallelize
    let output = work_dir.run_jj(["parallelize", "description(b)", "main"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 2 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // rebase -s
    let output = work_dir.run_jj(["rebase", "-s=main", "-d=@"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // rebase -b
    let output = work_dir.run_jj(["rebase", "-b=main", "-d=@"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit 77cee210cbf5 is immutable
    Hint: Could not modify commit: zsuskuln 77cee210 c
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 2 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // rebase -r
    let output = work_dir.run_jj(["rebase", "-r=main", "-d=@"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // resolve
    let output = work_dir.run_jj(["resolve", "-r=description(merge)", "file"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // restore -c
    let output = work_dir.run_jj(["restore", "-c=main"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // restore --into
    let output = work_dir.run_jj(["restore", "--into=main"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // split
    let output = work_dir.run_jj(["split", "-r=main"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // squash -r
    let output = work_dir.run_jj(["squash", "-r=description(b)"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit 72e1b68cbcf2 is immutable
    Hint: Could not modify commit: kkmpptxz 72e1b68c b
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 4 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // squash --from
    let output = work_dir.run_jj(["squash", "--from=main"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // squash --into
    let output = work_dir.run_jj(["squash", "--into=main"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // sign
    let output = work_dir.run_jj(["sign", "-r=main", "--config=signing.backend=test"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
    // unsign
    let output = work_dir.run_jj(["unsign", "-r=main"]);
    insta::assert_snapshot!(output, @r#"
    ------- stderr -------
    Error: Commit bcab555fc80e is immutable
    Hint: Could not modify commit: mzvwutvl bcab555f main | (conflict) merge
    Hint: Immutable commits are used to protect shared history.
    Hint: For more information, see:
          - https://jj-vcs.github.io/jj/latest/config/#set-of-immutable-commits
          - `jj help -k config`, "Set of immutable commits"
    Hint: This operation would rewrite 1 immutable commits.
    [EOF]
    [exit status: 1]
    "#);
}
