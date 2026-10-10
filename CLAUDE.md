# CLAUDE.md

## 编译与测试（test checkout）

禁止在 worktree（含主 checkout）里原地运行 cargo 编译。编译与测试只在 test checkout 进行，由 `~/.local/bin/cargo` 包装实现：

1. 从 `main` 开 linked worktree，在 worktree 里改代码。
2. 在 worktree 里直接执行 `cargo …`。包装会把 worktree 当前内容（含未提交改动和新文件）打成快照，切到 `~/program/worktree/slide-captcha-rs/test`（`test` 分支）里编译和测试；`cargo run` 编译完后回到 worktree 运行。不需要手动 commit，也不需要手动合并到 `test`。输出里的路径已改写回 worktree 路径。
3. 测试失败：回 worktree 修改，再执行同一条命令。
4. 验证通过后，按 merge-main 合入 `main`。

约束：

- 禁止在 `~/program/worktree/slide-captcha-rs/test` 里改代码或提交：每次编译前它会被 `checkout -f`、`clean -fd` 覆盖。
- 禁止删除 test worktree 或 `test` 分支。
- 禁止设置 `CARGO_TARGET_DIR` 或直接调用 `~/.cargo/bin/cargo` 绕开包装。
- 编译串行（并发 1）。看到 `[cargo] 共享 target 正被占用，排队中……` 时等待，不要 kill 进程。
- 编译需要、但被 gitignore 的文件，要列在仓库根的 `.test-checkout-include`（每行一个相对路径），包装会按内容同步到 test checkout。
