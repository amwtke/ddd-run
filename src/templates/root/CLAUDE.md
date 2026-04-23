# CLAUDE.md

> 本文件是 Claude Code 在本项目的**最高优先级约束**。
> 所有生成的代码、文档、测试必须符合本文件的规则。
> 本项目使用 `ddd-run` 搭建的 DDD + Superpowers harness。

## 项目定位
<请在此处填写项目一句话描述,例如:"会员积分管理系统的领域服务">

## 技术栈约定

> ⚠️ **本段未填充前,任何 skill(包括 Superpowers)不得产出实现代码。**
> 栈决策必须由 `superpowers:brainstorming` 驱动,在回答完 `/ddd-spec` 末尾"交给 Superpowers 的开放问题"后,由用户确认并**写回本段**,再进入 `writing-plans`。

决策必须覆盖:
- 语言 / 运行时
- 应用框架(Web / Service / CLI)
- 持久化方案(关系型 / 文档型 / 事件存储 / 内存)
- **范围:仅后端,还是前后端全栈?**若含前端,用什么框架
- 对外交互形态(REST / gRPC / GraphQL / CLI / 消息)
- 测试框架 / 构建工具

<!-- 填充示例(完成后删除本注释,替换为上面的条目):
- 语言:Java 17
- 框架:Spring Boot 3.x
- 持久化:MyBatis(仓储实现)/ JPA(可选)
- 辅助:Lombok(仅 DTO/VO),MapStruct(DTO ↔ Domain)
- 范围:仅后端服务
- 交互:REST
- 测试:JUnit 5 + AssertJ + Mockito
- 构建:Maven
-->

_待填充(由 `superpowers:brainstorming` 产出后写回本段)_

## 分层架构(强制)

```
┌──────────────────────────────────────────┐
│  Interfaces (REST Controller / DTO)       │  ← 只做序列化/反序列化
├──────────────────────────────────────────┤
│  Application (Application Service)        │  ← 编排/事务/发消息,无业务规则
├──────────────────────────────────────────┤
│  Domain (Aggregate / Entity / VO / DS)    │  ← 业务规则的唯一归属
├──────────────────────────────────────────┤
│  Infrastructure (Repository Impl / MQ)    │  ← 纯技术实现
└──────────────────────────────────────────┘
```

**依赖方向**:只能由上层依赖下层,**Domain 层不依赖任何其他层**(不得 import Spring 注解除 `@DomainEvent` 之类的自定义)。

## 强制规则(Hard Rules)

### R1. 战略层先行 + 技术栈先决策
任何新特性的实现顺序必须是:
```
/ddd-storm → /ddd-model(更新 DOMAIN.md)→ /ddd-spec
          → superpowers:brainstorming(若"## 技术栈约定"未填)
          → superpowers:writing-plans
          → superpowers:executing-plans(TDD)
          → superpowers:finishing-a-development-branch
```
**禁止跳过前面任一步直接写代码**。尤其:
- 进入 `writing-plans` 之前,本文件"## 技术栈约定"段必须已填(由 `brainstorming` 写回)。
- 如果用户要求跳步,请指出这会违反本项目的 harness 约定。

### R2. 术语一致性
所有代码命名必须引用 `DOMAIN.md` 中的 **Ubiquitous Language 表**。
- ✅ `class Member` / `class PointAccount`(与 DOMAIN.md 一致)
- ❌ `class User`(DOMAIN.md 里叫 Member)
- ❌ `class Account`(语义模糊,DOMAIN.md 明确是 PointAccount)

如发现代码中的命名与 DOMAIN.md 不一致,**停下来**,不要自作主张修改,而是询问用户:
> 代码中的 `X` 与 DOMAIN.md 中的 `Y` 不一致,是代码要改还是 DOMAIN.md 要改?

### R3. 富领域模型(禁止贫血)
聚合根必须封装业务行为:
- ✅ `order.addItem(product, quantity)` — 聚合根校验并修改自身
- ❌ `orderService.addItem(order, product, quantity)` — 规则写在 Service,聚合变成数据袋

Application Service 只能做这几件事:
1. 获取聚合根(从 Repository)
2. 调用聚合根的业务方法
3. 持久化(通过 Repository)
4. 发布领域事件
5. 处理事务边界

**任何 `if/else`、`for` 循环里包含业务判断的代码都必须在 Domain 层,不在 Application**。

### R4. 聚合边界
- 一个事务只能修改**一个**聚合实例
- 跨聚合协作用领域事件,不用同步调用
- 聚合之间只引用 ID,不持有对象

### R5. Repository 只对聚合根
- ✅ `MemberRepository`、`OrderRepository`
- ❌ `OrderItemRepository`、`AddressRepository`(VO)

### R6. TDD 节奏(由 Superpowers 执行)
进入实现阶段后,严格遵循 Superpowers 的 spec → test → code 节奏:
1. 一次只处理一个 spec
2. 先写测试,让它失败
3. 最小改动让测试通过
4. 重构
5. 进入下一个 spec

**禁止"一次性生成整套代码"**。如用户要求一次性生成,请指出这违反 harness 约定。

### R7. 包结构

> 下方示例为 Java / Spring 风格。若 `superpowers:brainstorming` 选定了其他栈,本节应由 Superpowers 替换为该栈对应的目录/模块约定(但分层含义不变:interfaces / application / domain / infrastructure)。

```
com.example.<module>/
├── interfaces/
│   ├── rest/                 # Controller
│   └── dto/                  # Request/Response
├── application/
│   ├── service/              # Application Service
│   └── event/                # Event Handler
├── domain/
│   ├── model/
│   │   ├── <aggregate>/      # 每个聚合一个包
│   │   └── shared/           # 共享 VO (Money, etc.)
│   ├── repository/           # Repository 接口
│   ├── service/              # Domain Service
│   └── event/                # Domain Event 定义
└── infrastructure/
    ├── persistence/          # Repository 实现(MyBatis Mapper)
    └── messaging/            # MQ 等
```

## 工作流总览

```
┌──────────────────────────────────────────────────────────────┐
│                      战略层(建模)                           │
│  业务需求 ─→ /ddd-storm ─→ /ddd-model ─→ DOMAIN.md(SSoT)   │
└──────────────────────────────────────────────────────────────┘
                              │
                              ↓
┌──────────────────────────────────────────────────────────────┐
│                  桥接层(切分 + 栈决策)                      │
│  DOMAIN.md + 用例 ─→ /ddd-spec ─→ docs/specs/spec-*.md       │
│                          ↓                                    │
│  superpowers:brainstorming ─→ 技术栈 / FE-BE 范围 / 交互     │
│                              形态(写回本文件 "## 技术栈")   │
│                          ↓                                    │
│  superpowers:writing-plans ─→ docs/superpowers/plans/*.md    │
└──────────────────────────────────────────────────────────────┘
                              │
                              ↓
┌──────────────────────────────────────────────────────────────┐
│                      战术层(实现)                           │
│  executing-plans ─→ TDD ─→ 测试 ─→ 实现 ─→ finishing-branch  │
└──────────────────────────────────────────────────────────────┘
```

## 修改 DOMAIN.md 的流程

`DOMAIN.md` 是领域模型的 Single Source of Truth,**不得随意修改**。

允许的修改路径:
1. 通过 `/ddd-model` 重新建模(推荐)
2. 在 `/ddd-spec` 过程中发现缺失,**停下来先修 DOMAIN.md 再继续**

禁止的修改路径:
- ❌ Superpowers 实现过程中擅自修改 DOMAIN.md
- ❌ 为了让代码通过测试而改 DOMAIN.md 的术语

## 代码质量底线
- 每个聚合必须有单元测试(覆盖不变式)
- 每个 Application Service 方法必须有集成测试
- 测试命名使用业务语言:`shouldRedeemPointsWhenBalanceIsSufficient`
- 禁止魔法数字(用 `Points.of(100)` 而非 `100`)
- 禁止 `public` 字段(除 `record` 组件)

---
*Generated by ddd-run. 本文件可根据项目实际情况调整,但不要删除"强制规则"部分。*
