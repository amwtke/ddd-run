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

## 分层架构(Bob 同心圆 4 环,详见 R7)

```
┌──────────────────────────────────────────┐
│  framework (Ring 4)                       │  ← Spring 装配 + 事务装饰器 + main
├──────────────────────────────────────────┤
│  adapter (Ring 3)                         │  ← REST Controller / Saga / Repository 实现 / ACL
├──────────────────────────────────────────┤
│  usecase (Ring 2)                         │  ← Interactor (POJO,零 Spring,零 SLF4J)
├──────────────────────────────────────────┤
│  domain (Ring 1)                          │  ← 实体 / 值对象 / 聚合根 / 领域事件(纯 Java)
└──────────────────────────────────────────┘
```

**依赖方向**:只能由外向内。`domain` 不 import 任何东西;`usecase` 只 import `domain`;`adapter` 可 import `usecase` 与 `domain`;`framework` 可 import 一切。

**关键铁律**(完整规则见 R7-R11):
- usecase 层**禁止**任何 Spring / Jakarta / SLF4J import 或注解
- 全工程**唯一**的 `@Transactional` 在 `shared.framework.transaction.TransactionalUseCaseDecorator`
- 领域事件 = 纯 Java record;聚合根登记,Outbox 发布,`@EventListener` 只在 `adapter/messaging/`

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
4. 登记领域事件(聚合根 `events.add(...)`,实际发布由 Repository 的 Outbox 完成,见 R10)
5. 处理事务边界

**任何 `if/else`、`for` 循环里包含业务判断的代码都必须在 Domain 层,不在 Application**。

**R3 补充(Clean Architecture)**:用例层(`usecase/**`)严禁出现任何注解或框架 import。
usecase 是纯 Java POJO,通过构造器注入端口接口,业务规则委托给聚合根。
日志需求通过 `LoggerPort` 抽象,不允许直接 `import org.slf4j.*`。

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

### R7. 包结构(4 环 Clean Architecture)

> 下方默认是 Java / Spring 风格。若 `superpowers:brainstorming` 选定了其他栈,目录命名按该栈调整,但**4 环依赖语义不变**。

每个聚合内部:
```
com.example.ddd.<aggregate>/
├── domain/                                Ring 1 — 实体 + 业务规则(零框架)
│   ├── <AggregateRoot>.java
│   ├── <Entity>.java
│   ├── <ValueObject>.java
│   ├── <AggregateRoot>Repository.java     端口接口,纯 Java
│   └── events/
│       └── <DomainEvent>.java             纯 record,零注解
├── usecase/                               Ring 2 — Interactor (POJO,零框架)
│   ├── <Command>Command.java
│   ├── <Command>UseCase.java              POJO,implements UseCase<C, R>
│   ├── <Command>Result.java               record,Controller 用,避免泄露 domain VO
│   └── port/
│       ├── <Gateway>.java                 出站端口(外部系统/ACL)
│       └── LoggerPort.java                仅在 usecase 需日志时
├── adapter/                               Ring 3 — 允许 Spring/JPA
│   ├── web/<Aggregate>Controller.java     入站 REST
│   ├── messaging/<X>EventHandler.java     入站 MQ/Saga
│   ├── persistence/
│   │   ├── <Aggregate>JpaEntity.java
│   │   ├── SpringData<Aggregate>Repo.java
│   │   └── Jpa<Aggregate>Repository.java  实现 domain 端口
│   ├── acl/<Gateway>HttpAcl.java          出站 ACL
│   └── logging/Slf4jLoggerAdapter.java    LoggerPort 实现
└── framework/                             Ring 4 — 允许全 Spring
    └── config/
        └── <Aggregate>UseCaseConfig.java  @Bean 装配
```

工程级共享包:
```
com.example.ddd.shared/
├── domain/DomainEvent.java                marker interface
├── usecase/UseCase.java                   通用 UseCase<C, R> 接口
└── framework/
    ├── transaction/TransactionalUseCaseDecorator.java   全工程唯一 @Transactional
    └── DddDemoApplication.java                          @SpringBootApplication main
```

**依赖方向**:只能由外向内。`domain` 不 import 任何东西;`usecase` 只 import `domain`;
`adapter` 可 import `usecase` 与 `domain`;`framework` 可 import 一切。

### R8. 装配规则(Spring 注解的位置)

1. `@Transactional` 在全工程**有且仅有一处**:`shared.framework.transaction.TransactionalUseCaseDecorator.execute()` 方法
2. Spring 注解(`@Service` / `@Component` / `@Repository` / `@Configuration` / `@Bean` / `@RestController` / `@EventListener` / `@Autowired` 等)**只允许**出现在 `adapter/**` 或 `framework/**`
3. 任何 usecase 都通过 `<aggregate>.framework.config.<Aggregate>UseCaseConfig` 中的 `@Bean` 注册,**必须**经 `TransactionalUseCaseDecorator` 包装(命令、查询统一,无例外)

### R9. 反模式硬清单(代码不得通过 review)

- ❌ `usecase/**/*.java` 出现任何 `import org.springframework.*` / `jakarta.persistence.*` / `jakarta.inject.*` / `org.slf4j.*`
- ❌ `usecase/**/*.java` 出现任何注解
- ❌ `domain/**/*.java` 出现任何框架 / Lombok / SLF4J import
- ❌ `@Transactional` 出现在 `shared.framework.transaction.TransactionalUseCaseDecorator` 之外
- ❌ `adapter/**` 之间横向 import(只能向内)
- ❌ `adapter/web/Controller` 直接 import `domain.*`(应通过 usecase 的 Command / Result 类型)
- ❌ Repository 实现放在 `framework/`(应在 `adapter/persistence/`)
- ❌ Saga handler 内部出现 `if/else` 业务分支(只能事件 → Command 翻译 + 调 usecase)
- ❌ usecase 跳过装饰器,Config 直接 `@Bean` 返回裸 POJO
- ❌ usecase 内出现 `Logger log = LoggerFactory.getLogger(...)`(必须用 `LoggerPort`)
- ❌ usecase 出现 `applicationEventPublisher.publishEvent(...)` 或同等手动发布(必须由聚合根登记 + Repository 写 Outbox)
- ❌ Domain Event 类带任何注解(`@DomainEvent` 自定义注解、Jackson 注解、Lombok 等都禁止)

### R10. Domain Event 边界

1. 事件类定义在 `domain/<aggregate>/events/`,纯 Java record,**禁止任何注解 / 框架 import**;用 marker interface `DomainEvent` 替代任何注解标记
2. 聚合根**登记**事件(`events.add(...)`),usecase **禁止**手动 publish
3. Outbox 写入在 `adapter/persistence/` 的 Repository 实现内,与聚合保存**同事务**(transactional outbox 模式)
4. Spring `ApplicationEventPublisher` / `@EventListener` / Kafka 客户端等基础设施**只允许**出现在 `adapter/messaging/`

### R11. ArchUnit 守卫

`src/test/java/architecture/CleanArchitectureTest.java` 是 R7-R10 的**机械执法者**。
不要删除已有规则;可在文件末尾追加项目特定规则。CI 必须运行该测试并对失败阻塞合并。

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
