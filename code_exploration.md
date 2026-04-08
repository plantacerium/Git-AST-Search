# 🕵️ Git AST Search: Code Exploration Catalog

Welcome to the **Code Exploration Catalog**. This document provides **88 high-impact AST patterns** designed to be used with the `/search` command in your TUI. These patterns leverage `ast-grep` to find structural code smells, security vulnerabilities, and architectural patterns across multiple languages.

---

## 📑 Table of Contents

1. [Fundamental & General](#1-fundamental--general)
2. [Rust Professional](#2-rust-professional)
3. [Modern JavaScript](#3-modern-javascript)
4. [TypeScript Safety](#4-typescript-safety)
5. [Pythonic Patterns](#5-pythonic-patterns)
6. [Go Concurrency & Errors](#6-go-concurrency---errors)
7. [Java Enterprise](#7-java-enterprise)
8. [Systems C/C++](#8-systems-cc)
9. [Security & Audit](#9-security--audit)
10. [Refactoring & Technical Debt](#10-refactoring--technical-debt)
11. [Testing & QA](#11-testing--qa)
12. [Git-AST-Search Internals](#12-git-ast-search-internals)

---

## 🛠️ 1. Fundamental & General
*Versatile patterns for quick discovery across many languages.*

| # | Name | Pattern | Description |
|---|---|---|---|
| 1 | **TODO Finder** | `// TODO: $$$` | Locate all pending tasks in comments. |
| 2 | **Empty Blocks** | `{ }` | Find empty function or control flow blocks. |
| 3 | **Hardcoded Strings** | `"$STR"` | Identify literal strings that might need internationalization. |
| 4 | **Return All** | `return $$$` | Trace all exit points in functions. |
| 5 | **Double Negation** | `!!$A` | Find "truthy" conversions that could be more explicit. |
| 6 | **Generic Logging** | `$LOG.log($$$)` | Locate logging statements across various libraries. |
| 7 | **Infinite Loops** | `while (true) { $$$ }` | Find potentially dangerous infinite loops. |
| 8 | **Generic If-True** | `if (true) { $$$ }` | Find redundant conditional blocks. |
| 9 | **Function Call** | `$F($$$)` | Find any function invocation. |
| 10 | **Class Definition** | `class $N { $$$ }` | Locate class declarations. |
| 11 | **If-Else Block** | `if ($C) { $$$ } else { $$$ }` | Identify branching logic. |
| 12 | **Try-Catch Block** | `try { $$$ } catch ($E) { $$$ }` | Find exception handling. |
| 13 | **Array Declaration** | `[$A, $B, $$$]` | Find array initializations. |
| 14 | **Object Literal** | `{ $K: $V, $$$ }` | Find object map declarations. |
| 15 | **Switch Statement** | `switch ($V) { $$$ }` | Find switch branching. |
| 16 | **While Loop** | `while ($C) { $$$ }` | Locate while loops. |
| 17 | **For Loop** | `for ($init; $cond; $inc) { $$$ }` | Find classic for loops. |
| 18 | **Arrow/Lambda Func** | `($$$) => { $$$ }` | Find anonymous functions. |
| 19 | **Variable Decl** | `let $X = $Y;` | Find variable declarations. |
| 20 | **Const Decl** | `const $X = $Y;` | Find constant declarations. |
| 21 | **Equality Check** | `$A == $B` | Find loose equality operators. |
| 22 | **Strict Equality** | `$A === $B` | Find strict equality operators. |
| 23 | **Assignment Check** | `if ($A = $B)` | Find assignment disguised as condition. |
| 24 | **Null Comparison** | `$A == null` | Find explicit null checks. |
| 25 | **Regex Literal** | `/$R/g` | Find regex literals. |
| 26 | **Ternary Operator** | `$C ? $A : $B` | Find inline conditionals. |
| 27 | **Break Statement** | `break;` | Locate loop escapes. |
| 28 | **Continue Stmt** | `continue;` | Locate loop iteration skips. |
| 29 | **Return Variable** | `return $V;` | Find variable returns. |
| 30 | **Return Call** | `return $F($$$);` | Find function call returns. |
| 31 | **Throw Error** | `throw $E;` | Find explicit error throwing. |
| 32 | **Async Function** | `async function $F($$$) { $$$ }` | Locate async functions. |
| 33 | **Await Call** | `await $P` | Find await resolution. |
| 34 | **Import Statement** | `import $$$ from '$M';` | Find module imports. |
| 35 | **Export Statement** | `export $$$;` | Find module exports. |
| 36 | **Require Call** | `require('$M')` | Find CommonJS requires. |
| 37 | **Module Exports** | `module.exports = $$$` | Find CommonJS exports. |
| 38 | **JSON Parse** | `JSON.parse($$$)` | Find JSON parsing. |
| 39 | **JSON Stringify** | `JSON.stringify($$$)` | Find JSON serialization. |
| 40 | **SetTimeout** | `setTimeout($$$)` | Find delayed executions. |
| 41 | **SetInterval** | `setInterval($$$)` | Find recurring executions. |
| 42 | **Fetch Call** | `fetch($$$)` | Find network requests. |
| 43 | **Math Random** | `Math.random()` | Find random usage. |
| 44 | **ParseInt** | `parseInt($$$)` | Find integer conversions. |
| 45 | **String Replace** | `$S.replace($$$)` | Find string replacements. |
| 46 | **String Split** | `$S.split($$$)` | Find string splits. |
| 47 | **Array Map** | `$A.map($$$)` | Find functional mapping. |
| 48 | **Array Filter** | `$A.filter($$$)` | Find functional filtering. |
| 49 | **Array Reduce** | `$A.reduce($$$)` | Find functional reduction. |
| 50 | **Array ForEach** | `$A.forEach($$$)` | Find functional iteration. |
| 51 | **Object Keys** | `Object.keys($$$)` | Find key extraction. |
| 52 | **Object Values** | `Object.values($$$)` | Find value extraction. |
| 53 | **Promise All** | `Promise.all($$$)` | Find promise aggregation. |
| 54 | **New Promise** | `new Promise($$$)` | Find promise creation. |
| 55 | **Date Now** | `Date.now()` | Find current timestamp. |
| 56 | **New Date** | `new Date($$$)` | Find date instantiations. |
| 57 | **Console Error** | `console.error($$$)` | Find error logging. |
| 58 | **Console Warn** | `console.warn($$$)` | Find warning logging. |
| 59 | **Console Info** | `console.info($$$)` | Find info logging. |
| 60 | **Process Env** | `process.env.$VAR` | Find env variable access. |
| 61 | **Try Finally** | `try { $$$ } finally { $$$ }` | Find finally cleanup. |
| 62 | **Yield Statement** | `yield $V;` | Locate generator yields. |
| 63 | **Generator Func** | `function* $F($$$) { $$$ }` | Locate generator functions. |
| 64 | **Spread syntax** | `[...$A]` | Find array spreading. |
| 65 | **Rest params** | `function $F(...$args)` | Find rest parameter usage. |
| 66 | **Typeof check** | `typeof $A === $S` | Locate type checking. |
| 67 | **Instanceof check** | `$A instanceof $B` | Locate instance checking. |
| 68 | **Delete operator** | `delete $A.$K` | Find object property deletion. |
| 69 | **Window global** | `window.$K` | Locate browser global access. |
| 70 | **Document object** | `document.$K` | Locate typical DOM access. |
| 71 | **Local Storage** | `localStorage.$K` | Locate local storage usage. |
| 72 | **Session Storage** | `sessionStorage.$K` | Locate session storage usage. |
| 73 | **Debugger Stmt** | `debugger;` | Find hardcoded breakpoints. |
| 74 | **Alert Dialog** | `alert($$$)` | Find annoying browser alerts. |
| 75 | **Confirm Dialog** | `confirm($$$)` | Find confirm prompts. |
| 76 | **Prompt Dialog** | `prompt($$$)` | Find user input prompts. |
| 77 | **Void Operator** | `void $A` | Locate void operator usage. |
| 78 | **Bitwise AND** | `$A & $B` | Find bitwise AND. |
| 79 | **Bitwise OR** | `$A | $B` | Find bitwise OR. |
| 80 | **Bitwise XOR** | `$A ^ $B` | Find bitwise XOR. |
| 81 | **Left Shift** | `$A << $B` | Find left shift. |
| 82 | **Right Shift** | `$A >> $B` | Find right shift. |
| 83 | **In Operator** | `$K in $O` | Find property check. |
| 84 | **With Statement** | `with ($A) { $$$ }` | Find dangerous with statements. |
| 85 | **String Literal** | '"$S"' | Find standard double-quote string |
| 86 | **Void Return** | `return;` | Find early void returns. |
| 87 | **Class Constructor** | `constructor($$$) { $$$ }` | Find class initializers. |
| 88 | **Static Method** | `static $F($$$) { $$$ }` | Find static class methods. |

---

## 🦀 2. Rust Professional
*Deep dives into memory safety and idiomatic Rust.*

| # | Name | Pattern | Description |
|---|---|---|---|
| 89 | **Unsafe Blocks** | `unsafe { $$$ }` | Audit all manual memory management and FFI. |
| 90 | **Risky Unwraps** | `$A.unwrap()` | Find potential panics from Option/Result. |
| 91 | **Panic! Macro** | `panic!($$$)` | Locate explicit program terminations. |
| 92 | **Expect Pattern** | `$A.expect($$$)` | View custom error messages for crashes. |
| 93 | **Ignored Results** | `let _ = $FUNC($$$);` | Find where function results are explicitly discarded. |
| 94 | **Todo Macro** | `todo!($$$)` | Find unimplemented features in the codebase. |
| 95 | **Manual Drops** | `drop($A)` | Trace explicit memory deallocations. |
| 96 | **Clone Calls** | `$A.clone()` | Find potential performance bottlenecks due to data copying. |
| 97 | **Match Statement** | `match $V { $$$ }` | Find match expressions. |
| 98 | **If Let** | `if let $P = $V { $$$ }` | Locate pattern matching via if let. |
| 99 | **While Let** | `while let $P = $V { $$$ }` | Locate pattern matching loops. |
| 100 | **Struct Decl** | `struct $N { $$$ }` | Find struct definitions. |
| 101 | **Enum Decl** | `enum $N { $$$ }` | Find enum definitions. |
| 102 | **Trait Decl** | `trait $N { $$$ }` | Find trait definitions. |
| 103 | **Impl Block** | `impl $N { $$$ }` | Find implementation blocks. |
| 104 | **Trait Impl** | `impl $T for $N { $$$ }` | Find trait implementations. |
| 105 | **Type Alias** | `type $N = $T;` | Find type aliases. |
| 106 | **Const Def** | `const $N: $T = $V;` | Find constant definitions. |
| 107 | **Static Def** | `static $N: $T = $V;` | Find static definitions. |
| 108 | **Box New** | `Box::new($$$)` | Locate box allocations. |
| 109 | **Rc New** | `Rc::new($$$)` | Locate reference counted allocations. |
| 110 | **Arc New** | `Arc::new($$$)` | Locate atomic ref-counted allocations. |
| 111 | **Mutex Lock** | `$M.lock().unwrap()` | Find mutex locking operations. |
| 112 | **RwLock Read** | `$M.read().unwrap()` | Find RwLock reading. |
| 113 | **RwLock Write** | `$M.write().unwrap()` | Find RwLock writing. |
| 114 | **Vec New** | `Vec::new()` | Locate vector initializations. |
| 115 | **Vec Macro** | `vec![$$$]` | Locate vec macro usage. |
| 116 | **String New** | `String::new()` | Locate string initializations. |
| 117 | **String From** | `String::from($$$)` | Locate string conversions from literals. |
| 118 | **ToString** | `$A.to_string()` | Locate implicit to_string calls. |
| 119 | **Format Macro** | `format!($$$)` | Find string formatting. |
| 120 | **Println Macro** | `println!($$$)` | Find standard prints. |
| 121 | **Eprint Macro** | `eprintln!($$$)` | Find stderr prints. |
| 122 | **Dbg Macro** | `dbg!($$$)` | Find debug macro usage. |
| 123 | **Result Ok** | `Ok($V)` | Find Result Ok instantiations. |
| 124 | **Result Err** | `Err($E)` | Find Result Err instantiations. |
| 125 | **Option Some** | `Some($V)` | Find Option Some instantiations. |
| 126 | **Option None** | `None` | Find Option None instantiations. |
| 127 | **Try operator** | `$F()?` | Find ? operator usage for error propagation. |
| 128 | **Unreachable** | `unreachable!($$$)` | Find unreachable panics. |
| 129 | **Unimplemented** | `unimplemented!($$$)` | Find unimplemented macro usages. |
| 130 | **Assert** | `assert!($$$)` | Find absolute assertions. |
| 131 | **Assert Eq** | `assert_eq!($$$)` | Find equality assertions. |
| 132 | **As Ref** | `$A.as_ref()` | Locate as_ref conversions. |
| 133 | **As Mut** | `$A.as_mut()` | Locate as_mut conversions. |
| 134 | **Into** | `$A.into()` | Locate implicit into conversions. |
| 135 | **Try Into** | `$A.try_into()` | Locate failable conversions. |
| 136 | **From Trait** | `impl From<$A> for $B` | Find From trait implementations. |
| 137 | **Into Trait** | `impl Into<$A> for $B` | Find Into trait implementations. |
| 138 | **Iterator Map** | `$I.map($$$)` | Locate iterator mappings. |
| 139 | **Iterator Filter** | `$I.filter($$$)` | Locate iterator filters. |
| 140 | **Iterator Collect** | `$I.collect()` | Locate iterator collections. |
| 141 | **Iterator Fold** | `$I.fold($$$)` | Locate iterator folds. |
| 142 | **Iterator ForEach** | `$I.for_each($$$)` | Locate iterator consumptions. |
| 143 | **Derive Clone** | `#[derive(Clone)]` | Find clone derivations. |
| 144 | **Derive Debug** | `#[derive($$$ Debug $$$)]` | Find debug derivations. |
| 145 | **Cfg Test** | `#[cfg(test)]` | Locate test modules. |
| 146 | **Test Attr** | `#[test]` | Locate unit test functions. |
| 147 | **Inline Attr** | `#[inline]` | Locate inline hints. |
| 148 | **Must Use Attr** | `#[must_use]` | Locate must_use attributes. |
| 149 | **Dead Code Attr** | `#[allow(dead_code)]` | Locate dead code suppressions. |
| 150 | **Async Block** | `async { $$$ }` | Find async logical blocks. |
| 151 | **Thread Spawn** | `thread::spawn($$$)` | Find thread spawns. |
| 152 | **Channel Decl** | `mpsc::channel()` | Find mpsc channel creations. |
| 153 | **Sender Send** | `$S.send($$$)` | Find channel communications. |
| 154 | **Receiver Recv** | `$R.recv()` | Find blocking receives. |
| 155 | **Receiver TryRecv** | `$R.try_recv()` | Find non-blocking receives. |
| 156 | **As Ptr** | `$A.as_ptr()` | Locate raw pointer extractions. |
| 157 | **As Mut Ptr** | `$A.as_mut_ptr()` | Locate raw mut pointer extractions. |
| 158 | **Ref Cell** | `RefCell::new($$$)` | Locate RefCell creations. |
| 159 | **Ref Borrow** | `$R.borrow()` | Locate dynamic immutable borrows. |
| 160 | **Ref Borrow Mut** | `$R.borrow_mut()` | Locate dynamic mutable borrows. |
| 161 | **Pin New** | `Pin::new($$$)` | Locate struct pinning. |
| 162 | **Phantom Data** | `PhantomData` | Locate phantom data structures. |
| 163 | **Default Impl** | `impl Default for $N` | Find default implementations. |
| 164 | **Drop Impl** | `impl Drop for $N` | Find drop resource cleanups. |
| 165 | **Display Impl** | `impl fmt::Display for $N` | Find custom string displays. |
| 166 | **Hash Map New** | `HashMap::new()` | Locate hash map creations. |
| 167 | **Hash Set New** | `HashSet::new()` | Locate hash set creations. |
| 168 | **Index Mut** | `impl IndexMut for $N` | Locate mutable indexing. |
| 169 | **Index** | `impl Index for $N` | Locate immutable indexing. |
| 170 | **Add Impl** | `impl Add for $N` | Find custom additions. |
| 171 | **Eq Impl** | `impl Eq for $N` | Find strict equality overrides. |
| 172 | **PartialEq Impl** | `impl PartialEq for $N` | Find partial equality overrides. |
| 173 | **Macro Rules** | `macro_rules! $N { $$$ }` | Find custom macro definitions. |
| 174 | **Generic Bounds** | `fn $F<T: $B>($$$)` | Find generic function bounds. |
| 175 | **Where Clause** | `where $T: $B` | Find bounded where clauses. |
| 176 | **Pub Crate** | `pub(crate) $N` | Find crate-visible items. |

---

## 🌐 3. Modern JavaScript
*Focus on performance, cleaner code, and DOM safety.*

| # | Name | Pattern | Description |
|---|---|---|---|
| 177 | **Console Logs** | `console.log($$$)` | Find debug prints that shouldn't be in production. |
| 178 | **Inner HTML** | `$A.innerHTML = $B` | Audit for potential XSS vulnerabilities. |
| 179 | **Eval Usage** | `eval($$$)` | Identify highly insecure dynamic code execution. |
| 180 | **Var Keyword** | `var $A = $B` | Find legacy variable declarations that should be `let`/`const`. |
| 181 | **Arrow Functions** | `($$$) => { $$$ }` | Locate modern functional style blocks. |
| 182 | **Async Closures** | `async ($$$) => { $$$ }` | Find asynchronous logic patterns. |
| 183 | **Promises Then** | `$A.then(($B) => { $$$ })` | Trace promise chains for readability. |
| 184 | **Template Literals** | `` `$$$` `` | Find dynamic string constructions. |

---

## 📘 4. TypeScript Safety
*Ensuring type integrity and catching "Type Holes".*

| # | Name | Pattern | Description |
|---|---|---|---|
| 185 | **Any Type** | `: any` | Locate where type safety is being bypassed. |
| 186 | **Non-Null Assert** | `$A!` | Find where compiler safety is manually overridden. |
| 187 | **Type Casting** | `$A as $B` | Audit force-casting of types. |
| 188 | **Interface Defs** | `interface $I { $$$ }` | Explore the data structures of the project. |
| 189 | **Readonly Props** | `readonly $P: $T` | Find immutable design patterns. |
| 190 | **Enum Usage** | `enum $E { $$$ }` | Locate constant enumerations. |
| 191 | **Generic Functions** | `fn $F<$T>($$$)` | Find reusable polymorphic logic. |
| 192 | **Namespace Usage** | `namespace $N { $$$ }` | Identify legacy TS module patterns. |

---

## 🐍 5. Pythonic Patterns
*Identifying common traps and idiomatic structures.*

| # | Name | Pattern | Description |
|---|---|---|---|
| 193 | **Mutable Default** | `def $F($A = []): $$$` | Catch the classic "shared list" default argument bug. |
| 194 | **Empty Except** | `except: pass` | Find silenced errors that hide bugs. |
| 195 | **Global Keyword** | `global $A` | Locate state pollution in global scope. |
| 196 | **List Comps** | `[$A for $B in $C]` | Find dense data processing logic. |
| 197 | **Context Managers** | `with $A as $B: $$$` | Trace resource management (files, DBs). |
| 198 | **Lambda Logic** | `lambda $A: $B` | Find anonymous functional snippets. |
| 199 | **Dunder Methods** | `def __$M__(self, $$$):` | Explore object-oriented hooks. |
| 200 | **Print Debug** | `print($$$)` | Find lingering debug output. |

---

## 🐹 6. Go Concurrency & Errors
*Mastering the "Go Way" of handling systems.*

| # | Name | Pattern | Description |
|---|---|---|---|
| 201 | **Error Checking** | `if err != nil { $$$ }` | Trace the standard error propagation flow. |
| 202 | **Ignored Errors** | `$V, _ := $F($$$)` | Find where critical errors are being ignored. |
| 203 | **Goroutines** | `go $F($$$)` | Identify concurrent execution points. |
| 204 | **Defer Pattern** | `defer $F($$$)` | Track cleanup logic and resource releases. |
| 205 | **Channel Make** | `make(chan $T, $S)` | Find inter-process communication setup. |
| 206 | **Select Stmt** | `select { $$$ }` | Audit complex concurrency multiplexing. |
| 207 | **Panic Call** | `panic($$$)` | Locate critical failure points. |
| 208 | **Context Pass** | `func $F(ctx context.Context, $$$)` | Trace request-scoped cancellation logic. |

---

## ☕ 7. Java Enterprise
*Standardizing patterns for large scale applications.*

| # | Name | Pattern | Description |
|---|---|---|---|
| 209 | **SysOut Print** | `System.out.println($$$)` | Identify non-logger based debugging. |
| 210 | **Generic Catch** | `catch (Exception $E) { $$$ }` | Find over-broad exception handling. |
| 211 | **Null Checks** | `if ($A == null) { $$$ }` | Trace defensive programming patterns. |
| 212 | **Synchronized** | `synchronized ($A) { $$$ }` | Audit thread-safety and lock points. |
| 213 | **Thread Sleep** | `Thread.sleep($$$)` | Find potential performance bottlenecks or wait hacks. |
| 214 | **Spring Autowire** | `@Autowired` | Explore dependency injection points. |
| 215 | **Stream Flow** | `$A.stream().$$$` | Locate functional-style data processing. |
| 216 | **Public Fields** | `public $T $F;` | Find encapsulated data violations in classes. |

---

## ⚙️ 8. Systems C/C++
*Low-level auditing for performance and safety.*

| # | Name | Pattern | Description |
|---|---|---|---|
| 217 | **Malloc Calls** | `malloc($$$)` | Audit heap memory allocations. |
| 218 | **Free Resource** | `free($$$)` | Match allocations with their cleanup. |
| 219 | **Strcpy Usage** | `strcpy($$$)` | Locate highly insecure string copies (use `strncpy`). |
| 220 | **Goto Labels** | `goto $L;` | Find "Spaghetti Code" navigation. |
| 221 | **Pragma Once** | `#pragma once` | Verify modern header guard usage. |
| 222 | **Delete Pointer** | `delete $P;` | Find C++ object deallocations. |
| 223 | **Printf Debug** | `printf($$$)` | Locate standard output logs. |
| 224 | **Nullptr Check** | `if ($P == nullptr)` | Find modern C++ null pointer safety. |

---

## 🔒 9. Security & Audit
*Finding the "Smoking Gun" in security reviews.*

| # | Name | Pattern | Description |
|---|---|---|---|
| 225 | **Hardcoded PW** | `password = "$PWD"` | Find potentially leaked credentials. |
| 226 | **MD5 Hashing** | `md5($$$)` | Identify weak cryptographic algorithms. |
| 227 | **Insecure Random** | `Math.random()` | Find non-cryptographic random numbers. |
| 228 | **CORS Wildcard** | `allowOrigin: "*"` | Audit overly permissive web security settings. |
| 229 | **Shell Spawn** | `spawn($$$)` | Trace external process execution (Injection risk). |
| 230 | **JWT Decode** | `jwt.decode($$$)` | Find where tokens are decoded without verification. |
| 231 | **SQL Injection** | `$DB.execute($Q + $I)` | Identify dangerous SQL dynamic construction. |
| 232 | **Base64 Secrets** | `atob($$$)` | Find where "security by obscurity" is used. |

---

## 🧹 10. Refactoring & Technical Debt
*Improving codebase health and maintainability.*

| # | Name | Pattern | Description |
|---|---|---|---|
| 233 | **Nested Ifs** | `if ($A) { if ($B) { if ($C) { $$$ } } }` | Identify deep nesting that needs flattening. |
| 234 | **Magic Numbers** | `return 42;` | Find literal values that should be constants. |
| 235 | **Dead Code** | `return; $$$` | Locate unreachable statements after a return. |
| 236 | **Complex Logic** | `if ($A && $B |  |
| 237 | **Duplicate Call** | `$A.get(); $A.get();` | Identify redundant computations or lookups. |
| 238 | **Large Params** | `fn $F($A, $B, $C, $D, $E, $F)` | Find functions with too many arguments. |
| 239 | **Constant Path** | `"/home/user/$$$"` | Locate machine-specific hardcoded paths. |
| 240 | **Empty Class** | `class $C { }` | Identify boilerplate or unfinished structures. |

---

## 🧪 11. Testing & QA
*Exploring the test suite architecture.*

| # | Name | Pattern | Description |
|---|---|---|---|
| 241 | **Assert Equals** | `assertEquals($A, $B)` | Trace equality assertions in tests. |
| 242 | **Mock Setup** | `mock($$$)` | Find where external dependencies are faked. |
| 243 | **Test Decorator** | `@Test` | List all entry points for the test runner. |
| 244 | **Describe Block** | `describe("$N", () => { $$$ })` | Explore test suite groupings (Jest/Mocha). |
| 245 | **Expect Match** | `expect($A).to.be($B)` | Find BDD-style assertions. |
| 246 | **Before Each** | `beforeEach(() => { $$$ })` | Trace test setup logic. |
| 247 | **Skipped Tests** | `it.skip($$$)` | Find ignored or broken tests in the suite. |
| 248 | **Benchmark** | `@Benchmark` | Locate performance measurement points. |

---

## 🏗️ 12. Git-AST-Search Internals
*Patterns specifically for exploring and auditing this repository.*

| # | Name | Pattern | Description |
|---|---|---|---|
| 249 | **UI Rendering** | `::render(f, $AREA, $$$)` | Find all component rendering calls and layouts. |
| 250 | **Nav Mode Change** | `self.nav_state.set_mode($MODE)` | Trace state transitions between Normal, Command, and Visual modes. |
| 251 | **Command Parsing** | `CommandParser::parse($INPUT)` | Identify where slash commands are decoded. |
| 252 | **Engine Messaging** | `Message::ResultFound($RES)` | Trace how search results flow from the background engine to the UI. |
| 253 | **Git Commit Lookup** | `repo.find_commit($OID)` | Find the low-level interaction with the libgit2 repository. |
| 254 | **Deduplication** | `visited_blobs.insert($ID)` | Audit the concurrent blob caching logic (DashMap). |
| 255 | **Parallelism** | `oids.par_chunks(100)` | Locate the Rayon-powered parallel scan entry point. |
| 256 | **Result Creation** | `SearchResult::new($$$)` | Trace where metadata (OID, Path, Line) is aggregated. |
| 257 | **Status Updates** | `self.status_message = Some($MSG)` | Find all points where the UI provides feedback to the user. |
| 258 | **Event Loop Logic** | `self.handle_key($KEY)` | Audit the centralized keyboard event orchestration. |
| 259 | **Textarea Input** | `self.textarea.input($KEY)` | Find where raw text input is passed to the search buffer. |
| 260 | **Autocomplete** | `self.autocomplete.suggest($B)` | Trace the heuristic logic behind the command completion system. |

---

> [!TIP]
> **Pro Tip:** Use the `--lang` flag in the TUI to narrow down these patterns to a specific language, e.g., `/search "unsafe { $$$ }" --lang rust`.
