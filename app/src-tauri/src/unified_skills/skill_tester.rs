//! Isolated QuickJS test runner for generated skills.
//!
//! Spins up a fresh `rquickjs` context (NOT the production engine), injects
//! mock bridge globals, loads the generated `index.js`, calls each tool with
//! empty args, and verifies that no exception is thrown and a value is returned.

use std::path::Path;

/// Result of a skill isolation test.
#[derive(Debug, Clone)]
pub struct TestResult {
    pub passed: bool,
    pub output: String,
    pub error: Option<String>,
}

/// Isolated skill tester.
pub struct SkillTester;

impl SkillTester {
    /// Run an isolated test of a skill in `skill_dir`.
    ///
    /// Steps:
    /// 1. Read `index.js` from `skill_dir`.
    /// 2. Create a fresh `rquickjs::AsyncRuntime` + `AsyncContext`.
    /// 3. Inject no-op mock globals for every bridge the skill might call.
    /// 4. Eval the skill source.
    /// 5. Call `init()` and `start()` if present.
    /// 6. For each tool in the `tools` array, call `tool.execute({})`.
    /// 7. Return `TestResult { passed: true }` if all pass without exception.
    pub async fn run_isolated(skill_dir: &Path) -> TestResult {
        let index_path = skill_dir.join("index.js");
        let js_source = match tokio::fs::read_to_string(&index_path).await {
            Ok(src) => src,
            Err(e) => {
                return TestResult {
                    passed: false,
                    output: String::new(),
                    error: Some(format!("Failed to read index.js: {e}")),
                };
            }
        };

        // Run the synchronous QuickJS work on the async runtime.
        // rquickjs AsyncRuntime is Send so we can use tokio::task::spawn_blocking
        // or just run inline — we use spawn_blocking to avoid blocking the executor
        // on a potentially long-running JS init/start sequence.
        let result = tokio::task::spawn_blocking(move || run_in_sync_context(&js_source)).await;

        match result {
            Ok(test_result) => test_result,
            Err(join_err) => TestResult {
                passed: false,
                output: String::new(),
                error: Some(format!("Test task panicked: {join_err}")),
            },
        }
    }
}

/// Synchronous entry point executed in a blocking thread.
/// Creates a single-threaded tokio runtime to drive the rquickjs async API.
fn run_in_sync_context(js_source: &str) -> TestResult {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            return TestResult {
                passed: false,
                output: String::new(),
                error: Some(format!("Failed to build tokio runtime: {e}")),
            };
        }
    };

    rt.block_on(run_async_test(js_source))
}

/// The async body of the test: creates the QuickJS context and exercises the skill.
async fn run_async_test(js_source: &str) -> TestResult {
    // --- Create a fresh QuickJS runtime (isolated from the production engine) ---
    let qjs_rt = match rquickjs::AsyncRuntime::new() {
        Ok(r) => r,
        Err(e) => {
            return TestResult {
                passed: false,
                output: String::new(),
                error: Some(format!("Failed to create QuickJS runtime: {e}")),
            };
        }
    };

    // Apply reasonable limits so a broken skill can't consume all memory.
    qjs_rt.set_memory_limit(32 * 1024 * 1024).await; // 32 MB
    qjs_rt.set_max_stack_size(256 * 1024).await; // 256 KB

    let ctx = match rquickjs::AsyncContext::full(&qjs_rt).await {
        Ok(c) => c,
        Err(e) => {
            return TestResult {
                passed: false,
                output: String::new(),
                error: Some(format!("Failed to create QuickJS context: {e}")),
            };
        }
    };

    // --- Phase 1: Inject mock globals ---
    let mock_globals = r#"
(function() {
    // db mock
    globalThis.db = {
        exec: function() {},
        get: function() { return null; },
        all: function() { return []; },
        kvSet: function() {},
        kvGet: function() { return null; }
    };
    // net mock — net.fetch is SYNCHRONOUS per skill contract
    globalThis.net = {
        fetch: function(url, opts) {
            return { status: 200, headers: {}, body: '{}' };
        }
    };
    // state mock
    globalThis.state = {
        set: function() {},
        get: function() { return null; },
        setPartial: function() {},
        delete: function() {},
        keys: function() { return []; }
    };
    // platform mock
    globalThis.platform = {
        os: function() { return 'macos'; },
        env: function(k) { return null; },
        notify: function() {}
    };
    // cron mock
    globalThis.cron = {
        register: function() {},
        unregister: function() {},
        list: function() { return []; }
    };
    // skills mock
    globalThis.skills = {
        list: function() { return []; },
        callTool: function() { return null; }
    };
    // log mock (some skills use log.info etc.)
    globalThis.log = {
        info: function() {},
        warn: function() {},
        error: function() {},
        debug: function() {}
    };
})();
"#;

    let inject_result = ctx
        .with(|js_ctx| {
            js_ctx
                .eval::<rquickjs::Value, _>(mock_globals.as_bytes())
                .map(|_| ())
                .map_err(|e| format_js_exception(&js_ctx, &e))
        })
        .await;

    if let Err(e) = inject_result {
        return TestResult {
            passed: false,
            output: String::new(),
            error: Some(format!("Mock globals injection failed: {e}")),
        };
    }

    // --- Phase 2: Eval the skill source ---
    let source = js_source.to_string();
    let eval_result = ctx
        .with(move |js_ctx| {
            js_ctx
                .eval::<rquickjs::Value, _>(source.as_bytes())
                .map(|_| ())
                .map_err(|e| format_js_exception(&js_ctx, &e))
        })
        .await;

    if let Err(e) = eval_result {
        return TestResult {
            passed: false,
            output: String::new(),
            error: Some(format!("Skill eval failed: {e}")),
        };
    }

    // Drive any pending micro-tasks.
    drive_jobs(&qjs_rt).await;

    // --- Phase 3: Call init() ---
    if let Err(e) = call_lifecycle_fn(&qjs_rt, &ctx, "init").await {
        return TestResult {
            passed: false,
            output: String::new(),
            error: Some(format!("init() failed: {e}")),
        };
    }

    // --- Phase 4: Call start() ---
    if let Err(e) = call_lifecycle_fn(&qjs_rt, &ctx, "start").await {
        return TestResult {
            passed: false,
            output: String::new(),
            error: Some(format!("start() failed: {e}")),
        };
    }

    // --- Phase 5: Count tools and call each tool.execute({}) ---
    let tool_count_result = ctx
        .with(|js_ctx| {
            let code = r#"(function() {
                if (typeof tools === 'undefined' || !Array.isArray(tools)) return 0;
                return tools.length;
            })()"#;
            js_ctx
                .eval::<i32, _>(code.as_bytes())
                .map_err(|e| format_js_exception(&js_ctx, &e))
        })
        .await;

    let tool_count = match tool_count_result {
        Ok(n) => n,
        Err(e) => {
            return TestResult {
                passed: false,
                output: String::new(),
                error: Some(format!("Failed to read tools array: {e}")),
            };
        }
    };

    let mut tool_outputs: Vec<String> = Vec::new();

    for idx in 0..tool_count {
        let call_code = format!(
            r#"(function() {{
                try {{
                    var tool = tools[{idx}];
                    if (!tool || typeof tool.execute !== 'function') {{
                        return JSON.stringify({{ ok: true, note: 'no execute fn' }});
                    }}
                    var result = tool.execute({{}});
                    // handle Promise
                    if (result && typeof result.then === 'function') {{
                        globalThis.__testToolDone_{idx} = false;
                        globalThis.__testToolError_{idx} = undefined;
                        globalThis.__testToolResult_{idx} = undefined;
                        result.then(
                            function(v) {{
                                // Treat {{error:...}} return values as failures
                                if (v && typeof v === 'object' && v.error) {{
                                    globalThis.__testToolError_{idx} = typeof v.error === 'string' ? v.error : JSON.stringify(v.error);
                                }} else {{
                                    globalThis.__testToolResult_{idx} = v;
                                }}
                                globalThis.__testToolDone_{idx} = true;
                            }},
                            function(e) {{
                                globalThis.__testToolError_{idx} = e && e.message ? e.message : String(e);
                                globalThis.__testToolDone_{idx} = true;
                            }}
                        );
                        return '__promise__';
                    }}
                    // Treat {{error:...}} return values as failures
                    if (result && typeof result === 'object' && result.error) {{
                        throw new Error(typeof result.error === 'string' ? result.error : JSON.stringify(result.error));
                    }}
                    return JSON.stringify({{ ok: true, result: result }});
                }} catch(e) {{
                    throw e;
                }}
            }})()"#,
            idx = idx
        );

        let call_result = ctx
            .with(move |js_ctx| {
                js_ctx
                    .eval::<String, _>(call_code.as_bytes())
                    .map_err(|e| format_js_exception(&js_ctx, &e))
            })
            .await;

        match call_result {
            Err(e) => {
                return TestResult {
                    passed: false,
                    output: tool_outputs.join("; "),
                    error: Some(format!("Tool[{idx}].execute() threw: {e}")),
                };
            }
            Ok(s) if s == "__promise__" => {
                // Drive promises until done
                let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
                loop {
                    drive_jobs(&qjs_rt).await;

                    let done = ctx
                        .with(move |js_ctx| {
                            let check =
                                format!("globalThis.__testToolDone_{idx} === true", idx = idx);
                            js_ctx.eval::<bool, _>(check.as_bytes()).unwrap_or(false)
                        })
                        .await;

                    if done {
                        // Check for error
                        let err_val = ctx
                            .with(move |js_ctx| {
                                let code = format!(
                                    r#"(function() {{
                                        var e = globalThis.__testToolError_{idx};
                                        return e ? String(e) : '';
                                    }})()"#,
                                    idx = idx
                                );
                                js_ctx
                                    .eval::<String, _>(code.as_bytes())
                                    .unwrap_or_default()
                            })
                            .await;

                        if !err_val.is_empty() {
                            return TestResult {
                                passed: false,
                                output: tool_outputs.join("; "),
                                error: Some(format!(
                                    "Tool[{idx}].execute() Promise rejected: {err_val}"
                                )),
                            };
                        }

                        let result_val = ctx
                            .with(move |js_ctx| {
                                let code = format!(
                                    r#"JSON.stringify(globalThis.__testToolResult_{idx})"#,
                                    idx = idx
                                );
                                js_ctx
                                    .eval::<String, _>(code.as_bytes())
                                    .unwrap_or_else(|_| "null".to_string())
                            })
                            .await;

                        tool_outputs.push(format!("tool[{idx}]: {result_val}"));
                        break;
                    }

                    if tokio::time::Instant::now() > deadline {
                        return TestResult {
                            passed: false,
                            output: tool_outputs.join("; "),
                            error: Some(format!(
                                "Tool[{idx}].execute() Promise timed out after 10s"
                            )),
                        };
                    }

                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                }
            }
            Ok(s) => {
                tool_outputs.push(format!("tool[{idx}]: {s}"));
            }
        }
    }

    TestResult {
        passed: true,
        output: if tool_outputs.is_empty() {
            format!("All {} tool(s) passed", tool_count)
        } else {
            tool_outputs.join("; ")
        },
        error: None,
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Drive all pending QuickJS micro-tasks / Promise jobs.
async fn drive_jobs(rt: &rquickjs::AsyncRuntime) {
    loop {
        let has_more = rt.is_job_pending().await;
        if !has_more {
            break;
        }
        if rt.execute_pending_job().await.is_err() {
            break;
        }
    }
}

/// Call a lifecycle function (init / start) if it exists, handling Promises.
async fn call_lifecycle_fn(
    rt: &rquickjs::AsyncRuntime,
    ctx: &rquickjs::AsyncContext,
    name: &str,
) -> Result<(), String> {
    let name = name.to_string();
    let is_promise = ctx
        .with(move |js_ctx| {
            let code = format!(
                r#"(function() {{
                    var fn = globalThis.{name};
                    if (typeof fn !== 'function') return '0';
                    var result = fn();
                    if (result && typeof result.then === 'function') {{
                        globalThis.__lifecycleDone = false;
                        globalThis.__lifecycleError = undefined;
                        result.then(
                            function() {{ globalThis.__lifecycleDone = true; }},
                            function(e) {{
                                globalThis.__lifecycleError = e && e.message ? e.message : String(e);
                                globalThis.__lifecycleDone = true;
                            }}
                        );
                        return '1';
                    }}
                    return '0';
                }})()"#,
                name = name
            );
            js_ctx
                .eval::<String, _>(code.as_bytes())
                .map_err(|e| format_js_exception(&js_ctx, &e))
        })
        .await?;

    if is_promise != "1" {
        return Ok(());
    }

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        drive_jobs(rt).await;

        let done = ctx
            .with(|js_ctx| {
                js_ctx
                    .eval::<bool, _>(b"globalThis.__lifecycleDone === true")
                    .unwrap_or(false)
            })
            .await;

        if done {
            let err = ctx
                .with(|js_ctx| {
                    let code = r#"(function() {
                        var e = globalThis.__lifecycleError;
                        return e ? String(e) : '';
                    })()"#;
                    js_ctx
                        .eval::<String, _>(code.as_bytes())
                        .unwrap_or_default()
                })
                .await;

            return if err.is_empty() { Ok(()) } else { Err(err) };
        }

        if tokio::time::Instant::now() > deadline {
            return Err("Lifecycle function timed out after 10s".to_string());
        }

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

/// Extract a human-readable error message from a QuickJS exception.
fn format_js_exception(js_ctx: &rquickjs::Ctx<'_>, err: &rquickjs::Error) -> String {
    if !err.is_exception() {
        return format!("{err}");
    }
    let exception = js_ctx.catch();
    if let Some(obj) = exception.as_object() {
        let message: String = obj.get::<_, String>("message").unwrap_or_default();
        let stack: String = obj.get::<_, String>("stack").unwrap_or_default();
        if !message.is_empty() {
            return if stack.is_empty() {
                message
            } else {
                format!("{message}\n{stack}")
            };
        }
    }
    if let Ok(s) = exception.get::<String>() {
        return s;
    }
    format!("{err}")
}
