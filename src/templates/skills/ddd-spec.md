---
name: ddd-spec
description: |
  触发条件:用户输入 /ddd-spec 加用例名(例如 /ddd-spec 会员兑换积分)。
  读取项目根目录的 DOMAIN.md,为指定用例生成一份 Superpowers 可直接消化的 spec 文档,
  严格使用 DOMAIN.md 中的 Ubiquitous Language 术语,包含用例描述、前置/后置条件、
  业务规则、测试场景(Given-When-Then)和一条明确的"交给 Superpowers 继续"的指令。
  这个 skill 是 DDD 建模阶段与 Superpowers 实现阶段的**桥梁**。
  当用户说"生成 spec"、"写用例描述"、"准备给 Superpowers 的输入"、"把这个用例写清楚"
  时也应触发此技能。
---

# DDD → Superpowers Spec Bridge Skill

## 触发
用户使用 `/ddd-spec <用例名>` 命令。前置条件:项目根目录必须存在 `DOMAIN.md`,
否则提示用户先运行 `/ddd-model`。

## 目标
把一个**业务用例**转化成一份 Superpowers 可直接消化的 spec 文档,
这份文档满足三个强约束:

1. **术语锚定**:所有命名必须来自 `DOMAIN.md` 的 Ubiquitous Language 表
2. **测试友好**:包含 Given-When-Then 场景,Superpowers 可直接转成测试
3. **边界清晰**:明确本 spec 影响哪个聚合,禁止隐式跨聚合

## 工作流

### Step 1:读入 DOMAIN.md
提取:
- Ubiquitous Language 术语表
- 相关聚合的不变式(Invariants)
- 相关聚合发出的领域事件

### Step 2:定位用例归属
判断本用例属于哪个聚合。如果跨聚合,**停下来提醒用户**:
> ⚠️ 这个用例似乎涉及聚合 A 和聚合 B。根据 DDD 原则,一个 spec 应只影响一个聚合。
> 建议拆成两个 spec:一个写 A 的本地操作,一个写 B 响应 A 事件的操作。

### Step 3:产出 spec

**严格按此模板产出**(写入 `docs/specs/spec-<序号>-<slug>.md`):

```markdown
# Spec: <用例名>

> 本 spec 由 /ddd-spec 生成,所有术语锚定到 DOMAIN.md。
> 实现阶段交给 Superpowers 的 TDD 流程驱动。

## 归属
- **聚合**:<AggregateRoot 名>(来自 DOMAIN.md § 3)
- **分层**:Application Service 编排 + Aggregate 内部实现业务规则

## 用例描述
<一段自然语言描述,必须只使用 DOMAIN.md 中的术语>

## 参与者
- <Actor>:<角色>

## 前置条件
- <用业务语言描述,如"会员账户存在且处于激活状态">
- ...

## 后置条件(成功路径)
- <如"积分账户余额减少 N 点,产生 PointRedeemed 事件">
- ...

## 业务规则(Invariants)
引用 DOMAIN.md 中该聚合的不变式,并补充本用例特有的规则:
- **INV-<n>**(来自 DOMAIN.md):<规则>
- **RULE-本用例-1**:<本用例特有的规则>

## 测试场景(Given-When-Then)

### 场景 1:<成功路径描述>
- **Given** <初始状态,用 DOMAIN.md 术语>
- **When** <触发的命令>
- **Then**
  - <状态变化>
  - <产生的领域事件>

### 场景 2:<边界/异常路径>
- **Given** ...
- **When** ...
- **Then** 抛出 `<DomainException>`

### 场景 3:<并发/幂等场景,如适用>
- ...

## 接口约定(初稿,可被 Superpowers 细化)

### Command
```java
// Application Service 层入口
record <CommandName>(
    <UbiquitousLanguageType> <field>,
    ...
) {}
```

### 聚合根方法
```java
// Aggregate Root 暴露的业务方法
public <ReturnType> <methodName>(<params>) {
    // 规则实现;禁止贫血
}
```

## 禁止项(Guardrails for Superpowers)
在实现此 spec 时,Superpowers 必须遵守:
- ❌ 不得把业务规则写进 Controller 或 Application Service
- ❌ 不得跨聚合直接调用(跨聚合必须通过领域事件)
- ❌ 不得修改 `DOMAIN.md` 之外的术语(命名必须一致)
- ❌ 不得绕过聚合根直接修改内部实体
- ✅ 必须先写测试(TDD),再写实现
- ✅ 必须为每个测试场景写一个测试用例

## 交给 Superpowers 的开放问题(技术实施层面)

本 spec **不回答**以下问题,留给 `superpowers:brainstorming` 在进入 `writing-plans` 之前回答,并把决策写回 `CLAUDE.md` 的"## 技术栈约定"段:

- **语言 / 运行时**:Java / Kotlin / Go / Rust / Node / Python / ... ?
- **应用框架**:Spring Boot / Ktor / Axum / Express / FastAPI / ... ?
- **持久化**:关系型 / 文档型 / 事件存储 / 内存?schema 由谁生成?
- **范围**:仅后端服务,还是前后端全栈?若含前端,用什么框架?
- **对外交互形态**:REST / gRPC / GraphQL / CLI / 消息?
- **测试框架 / 构建工具**
- **部署形态**:单体 / 模块化单体 / 微服务?
- **非功能约束**:并发量级、延迟预算、数据规模?

> 首次进入实现阶段时必答;若 `CLAUDE.md` 的技术栈段已填,则跳过并直接进入 `writing-plans`。

## 下一步

把本 spec 交给 Superpowers 全流程:

1. **`superpowers:brainstorming`**(首次):回答上面"交给 Superpowers 的开放问题",把技术栈 / 范围 / 交互形态决策写回 `CLAUDE.md`
2. **`superpowers:writing-plans`**:基于 spec + 已确定栈,产出分步实施计划
3. **`superpowers:executing-plans`** + TDD:先写失败测试 → 最小改动通过 → 重构
4. **`superpowers:finishing-a-development-branch`**:验证全绿 → 合并/开 PR

执行建议:
> 请 Superpowers 读取 `docs/specs/spec-<n>-<slug>.md`。若 `CLAUDE.md` 技术栈段未填,先启动 brainstorming 决定技术栈与范围并写回;否则直接进入 writing-plans。
> 实现过程中如发现 DOMAIN.md 中有不一致或缺失的术语,停下来回到 `/ddd-model` 修正。
```

### Step 4:回写 DOMAIN.md(仅当必要)
如果本用例暴露了 DOMAIN.md 中缺失的术语或规则,**停下来询问用户**是否先更新 DOMAIN.md。
**不要擅自修改 DOMAIN.md**。

## 与其他 skill 的衔接

```
  /ddd-storm ──→ docs/ddd/01-event-storming-*.md
                         ↓
  /ddd-model ──→ DOMAIN.md(Single Source of Truth)
                         ↓
  /ddd-spec  ──→ docs/specs/spec-*.md
                         ↓
  Superpowers ─→ 测试 → 实现
```

## 反模式(必须拒绝)

❌ spec 里出现 DOMAIN.md 之外的新术语(必须先更新 DOMAIN.md)
❌ 一个 spec 跨多个聚合(必须拆分)
❌ 只有 happy path,没有异常/边界场景
❌ Given-When-Then 里使用了技术语言("调用 API"、"写入数据库")而非业务语言

## 文件落位

- 产出:`docs/specs/spec-<自增序号>-<slug>.md`
- 如果目录不存在,先创建
