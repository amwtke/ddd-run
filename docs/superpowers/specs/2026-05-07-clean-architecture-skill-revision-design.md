# Clean Architecture Skill Revision · 设计文档

> 状态:设计已定稿,待用户审阅 → 转交 `superpowers:writing-plans` 落地。
> 日期:2026-05-07
> 作者:与 amwtke 共同 brainstorm 产出
> 适用仓库:`/Users/xiaojin/workspace/ddd-run`

## 1. 背景

`ddd-run` 是一个 Rust CLI,通过 `init` 命令向目标项目注入三个 skill(`ddd-storm` / `ddd-model` / `ddd-spec`)和两个根文档模板(`CLAUDE.md` / `DOMAIN.md`),目的是约束 Claude Code 在 DDD + Superpowers 流程下产出"战略层先行 + 严格 DDD"的代码。

**已观察到的问题**:按当前模板生成的样例项目 `../ddd-springboot-demo` 中,Application Service 直接使用了 `@Service` / `@Transactional`,Saga handler 使用 `@Component` + `@EventListener`(详见 `OrderApplicationService.java:8-9, 32, 52`、`InventoryApplicationService.java:8-9, 14, 31`、`StockEventHandler.java:8-9, 26, 37`)。这违反 Bob Martin 的 Clean Architecture:**用例层(Application/UseCase)应当是纯粹的业务编排,不应依赖任何框架(Spring、JPA、SLF4J 等)**。

domain 层目前是干净的(已确认 `order/domain/`、`inventory/domain/` 中无任何 framework import),问题面集中在 application 层。

## 2. 目标与非目标

### 目标
- 修订 `ddd-run` 的 skill 模板与根文档模板,使 `ddd-run init` 之后产出的新项目**严格遵守 Bob 同心圆 4 层向内依赖**
- 用例层做到"离了 Spring 也能在 main()/JUnit 里跑"的金标准
- 提供 ArchUnit 模板把规则机械化执行,避免人工 review 漏检
- 在不破坏现有 ddd-storm / DOMAIN.md 的前提下完成

### 非目标
- **不**重构 `ddd-springboot-demo` 自身使其合规(独立任务)
- **不**触动 `ddd-storm.md`(纯业务概念,与代码结构无关)
- **不**触动 `DOMAIN.md`(模型产出物,与代码结构无关)
- **不**为非 JVM 栈(Rust / Go / Node)定制 ArchUnit 替代方案(将来扩展)

## 3. 架构决策(已锁定)

### 3.1 风格:Bob 同心圆 4 层

```
domain/        Ring 1 — 实体 + 业务规则       (零框架)
usecase/       Ring 2 — Interactor (POJO)     (零框架,包括 SLF4J)
adapter/       Ring 3 — Controller / Saga / Gateway impl  (允许 Spring/JPA)
framework/     Ring 4 — Spring 装配 + 事务 + main          (允许全 Spring)
```

依赖方向:**只能由外向内**。`domain` 不 import 任何东西;`usecase` 只 import `domain`;`adapter` 可 import `usecase` 与 `domain`;`framework` 可 import 一切。

### 3.2 装配方式:纯 POJO + `framework/` 集中 `@Configuration`

每个 usecase 是普通 Java 类,**零注解、零 `import org.springframework.*` / `jakarta.*`**。
Spring 通过 `framework/config/<Aggregate>UseCaseConfig.java` 中的 `@Bean` 方法装配:
1. 构造纯 POJO usecase
2. 用 `TransactionalUseCaseDecorator` 包裹
3. 以 `UseCase<C, R>` 接口类型注册进容器

### 3.3 事务:T2-β,装饰器 + `@Transactional`

- 全工程**有且仅有一处** `@Transactional`,位于 `framework/transaction/TransactionalUseCaseDecorator.java`
- 装饰器实现 `UseCase<C, R>` 接口,委托内部 POJO usecase
- Spring AOP 自动织入事务代理
- **所有 usecase**(命令 + 查询)统一走装饰器(决策 4.2.ii)

### 3.4 Saga / 领域事件订阅:`adapter/messaging/`

- `@EventListener` handler 在 adapter 层
- handler 只做"事件 → Command 翻译 + 调用 usecase",**不允许 if/else 业务分支**

### 3.5 Repository:接口在 domain,实现在 adapter

- `domain/<Aggregate>Repository.java` — 端口接口,纯 Java
- `adapter/persistence/Jpa<Aggregate>Repository.java` — 实现接口,持有 Spring Data 接口
- `adapter/persistence/<Aggregate>JpaEntity.java` — JPA 映射,与 domain 模型分离

> **关于接口归属的折衷**:Bob 严派会把 Repository 接口放在 `usecase/port/out/`(出站端口归用例)。本设计保留 DDD 传统(接口在 domain),理由:① demo 现状已如此,改动小;② DDD 文献和实践一致性;③ 在 4 环依赖里,domain 里的接口既可被 usecase 使用,也可被 adapter 实现,语义无冲突。如果未来项目偏好 Bob 严派,可由 brainstorming 决定迁移到 usecase/port/out/,目录结构和 ArchUnit 规则相应调整。

### 3.6 SLF4J 边界:usecase 严禁

- `usecase/**/*.java` 禁止 `import org.slf4j.*`
- 若 usecase 需要日志,定义 `usecase/port/LoggerPort.java` 抽象,实现在 `adapter/logging/Slf4jLoggerAdapter.java`
- 决策依据:坚持"用例层离了 Spring + 离了任何外部库也能跑"的金标准(决策 4.2.i 严派)

### 3.7 Domain Event 生命周期

Bob 4 环并不否定 Domain Event;相反,**事件是跨聚合协作的唯一机制**(因为禁止跨聚合事务、禁止跨聚合直接调用)。事件生命周期的每个阶段被精确分配到对应环:

| 阶段 | 位置 | 备注 |
|---|---|---|
| **事件类定义** `OrderCreated` / `StockReserved` | `domain/<aggregate>/events/` | 纯 Java record,**零注解**,零框架 import |
| **聚合根登记事件** | `domain/<AggregateRoot>.java` 方法内 | 聚合根维护 `List<DomainEvent>`,规则触发时 `events.add(...)` |
| **usecase 调聚合根** | `usecase/<Cmd>UseCase.java` | usecase **从不**手动 `publish`,只调聚合根 + Repository |
| **Repository.save 写 Outbox** | `adapter/persistence/Jpa<Aggregate>Repository.java` | `save` 时 `pullEvents()` → 写 outbox 表(与聚合 update 同事务) |
| **Outbox publisher 广播** | `adapter/messaging/OutboxPublisher.java` | `@Scheduled` 或 transactional event listener,调 Kafka / Spring `ApplicationEventPublisher` |
| **Saga 订阅** `@EventListener on(...)` | `adapter/messaging/<X>EventHandler.java` | 把事件翻译成 Command,调下一个 usecase |
| **Saga 调到的 usecase** | `usecase/<NextCmd>UseCase.java` | 仍然纯 POJO,被 `TransactionalUseCaseDecorator` 装饰 |

事件类示例:
```java
// domain/order/events/OrderCreated.java —— 完整内容
package com.example.ddd.order.domain.events;

import java.time.Instant;

public record OrderCreated(
    String orderId,
    String customerId,
    Instant occurredAt
) implements DomainEvent {}
```

聚合根登记事件示例(对照,**usecase 不接触**):
```java
// domain/Order.java
private final List<DomainEvent> events = new ArrayList<>();
public void confirm() {
    this.status = CONFIRMED;
    events.add(new OrderConfirmed(this.id.value()));   // 只登记,不发布
}
public List<DomainEvent> pullEvents() { /* return + clear */ }
```

## 4. 标准目录布局(每个聚合)

```
com.example.ddd.<aggregate>/
├── domain/
│   ├── <AggregateRoot>.java
│   ├── <Entity>.java
│   ├── <ValueObject>.java
│   ├── <AggregateRoot>Repository.java       ← 端口接口(纯 Java)
│   └── events/
│       └── <DomainEvent>.java
├── usecase/
│   ├── <Command>Command.java                ← record,纯 Java
│   ├── <Command>UseCase.java                ← POJO Interactor
│   └── port/
│       ├── <Gateway>.java                   ← 出站端口(外部系统/ACL)
│       └── LoggerPort.java                  ← 仅在用例需要日志时
├── adapter/
│   ├── web/
│   │   └── <Aggregate>Controller.java       ← 入站 REST 适配器
│   ├── messaging/
│   │   └── <X>EventHandler.java             ← 入站 MQ/Saga 适配器
│   ├── persistence/
│   │   ├── <Aggregate>JpaEntity.java
│   │   ├── SpringData<Aggregate>Repo.java   ← Spring Data 接口
│   │   └── Jpa<Aggregate>Repository.java    ← 实现 domain 端口
│   ├── acl/
│   │   └── <Gateway>HttpAcl.java            ← 出站 ACL/HTTP 适配器
│   └── logging/
│       └── Slf4jLoggerAdapter.java          ← LoggerPort 实现(仅当需要)
└── framework/
    └── config/
        └── <Aggregate>UseCaseConfig.java    ← 该聚合的 @Bean 装配
```

### 工程级共享包

```
com.example.ddd.shared.framework/
├── transaction/
│   └── TransactionalUseCaseDecorator.java   ← 全工程唯一 @Transactional
├── usecase/
│   └── UseCase.java                         ← 通用 UseCase<C,R> 接口
└── DddDemoApplication.java                  ← @SpringBootApplication main
```

## 5. 范例:CreateOrder 完整链路

### 5.1 Ring 1 · domain
```java
// domain/OrderRepository.java —— 端口接口
public interface OrderRepository {
    void save(Order order);
    Optional<Order> findById(OrderId id);
}
```

### 5.2 Ring 2 · usecase(零 Spring,零 SLF4J)
```java
// usecase/CreateOrderUseCase.java
package com.example.ddd.order.usecase;

import com.example.ddd.order.domain.*;
import com.example.ddd.order.usecase.port.PricingGateway;
import com.example.ddd.shared.usecase.UseCase;

public class CreateOrderUseCase implements UseCase<CreateOrderCommand, OrderId> {

    private final OrderRepository orderRepo;
    private final PricingGateway pricing;

    public CreateOrderUseCase(OrderRepository orderRepo, PricingGateway pricing) {
        this.orderRepo = orderRepo;
        this.pricing = pricing;
    }

    @Override
    public OrderId execute(CreateOrderCommand cmd) {
        var customerId = CustomerId.of(cmd.customerId());
        var skuIds = cmd.lines().stream().map(l -> SkuId.of(l.skuId())).toList();
        var prices = pricing.fetchPrices(skuIds);
        var items = cmd.lines().stream()
            .map(l -> OrderItem.newItem(SkuId.of(l.skuId()), l.quantity(),
                                        prices.get(SkuId.of(l.skuId()))))
            .toList();
        Order order = Order.create(customerId, items);
        orderRepo.save(order);
        return order.id();
    }
}
```

### 5.3 Ring 3 · adapter
```java
// adapter/persistence/JpaOrderRepository.java —— 实现 domain 端口
@Repository
class JpaOrderRepository implements OrderRepository {
    private final SpringDataOrderRepo data;
    JpaOrderRepository(SpringDataOrderRepo data) { this.data = data; }
    @Override public void save(Order o) { data.save(OrderJpaEntity.from(o)); }
    @Override public Optional<Order> findById(OrderId id) {
        return data.findById(id.value()).map(OrderJpaEntity::toDomain);
    }
}

// adapter/web/OrderController.java —— 入站
// 注意:Controller 只 import usecase 包,严禁 import domain.*
// 因此 usecase 必须返回纯字符串/record 结果类型,不能返回 OrderId 这类 domain VO
@RestController @RequestMapping("/orders")
class OrderController {
    private final UseCase<CreateOrderCommand, CreateOrderResult> createOrder;
    OrderController(UseCase<CreateOrderCommand, CreateOrderResult> createOrder) {
        this.createOrder = createOrder;   // 注入装饰过的 Bean
    }
    @PostMapping
    public CreateOrderResult create(@RequestBody CreateOrderRequest req) {
        return createOrder.execute(req.toCommand());
    }
}

// usecase/CreateOrderResult.java —— 返回类型定义在 usecase 层,纯 Java
public record CreateOrderResult(String orderId) {}
```

### 5.4 Ring 4 · framework
```java
// shared/framework/transaction/TransactionalUseCaseDecorator.java —— 全工程唯一 @Transactional
public class TransactionalUseCaseDecorator<C, R> implements UseCase<C, R> {
    private final UseCase<C, R> inner;
    public TransactionalUseCaseDecorator(UseCase<C, R> inner) { this.inner = inner; }
    @Override
    @Transactional
    public R execute(C cmd) { return inner.execute(cmd); }
}

// order/framework/config/OrderUseCaseConfig.java —— 装配点
@Configuration
class OrderUseCaseConfig {
    @Bean
    UseCase<CreateOrderCommand, OrderId> createOrderUseCase(
            OrderRepository repo, PricingGateway pricing) {
        return new TransactionalUseCaseDecorator<>(
            new CreateOrderUseCase(repo, pricing));
    }
    // ... 其他 usecase 同样 ...
}
```

### 5.5 运行时调用链
```
HTTP POST /orders
  → OrderController.create                  [adapter/web]
  → UseCase.execute (Spring AOP 代理拦截)
  → TransactionalUseCaseDecorator.execute  ← 开启事务  [framework]
  → CreateOrderUseCase.execute             ← 纯 POJO   [usecase]
      ├─ pricing.fetchPrices               → PricingHttpAcl  [adapter/acl]
      ├─ Order.create                      → 聚合根     [domain]
      └─ orderRepo.save                    → JpaOrderRepository  [adapter/persistence]
  → 装饰器返回 ← 事务提交
  → Controller 返回 200
```

## 6. CLAUDE.md 模板修订

### 6.1 R3(强制规则·富领域模型)— 补充
原文末尾追加:
> **R3 补充**:用例层(`usecase/**`)严禁出现任何注解或框架 import。usecase 是纯 Java POJO,通过构造器注入端口接口,业务规则委托给聚合根。

### 6.2 R7(包结构)— 整段重写
替换为本文档 §4 的"标准目录布局"(分按聚合的内层 + 工程级共享包)。
保留原 R7 注释:**"若 brainstorming 选定其他栈,目录命名按该栈调整,但 4 环依赖语义不变"**。

### 6.3 R8(新增·装配规则)
> 1. `@Transactional` 在全工程**有且仅有一处**:`shared.framework.transaction.TransactionalUseCaseDecorator.execute()` 方法
> 2. Spring 注解(`@Service` / `@Component` / `@Repository` / `@Configuration` / `@Bean` / `@RestController` / `@EventListener` / `@Autowired` 等)只允许出现在 `adapter/**` 或 `framework/**`
> 3. 任何 usecase 都通过 `framework/config/<Aggregate>UseCaseConfig.java` 中的 `@Bean` 注册,**必须**经 `TransactionalUseCaseDecorator` 包装(命令、查询统一,无例外)

### 6.4 R9(新增·反模式硬清单)
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

### 6.5 R10(新增·Domain Event 边界)
1. 事件类定义在 `domain/<aggregate>/events/`,纯 Java record,**禁止任何注解 / 框架 import**(包括自定义的 `@DomainEvent` 类型注解 —— 用 marker interface `DomainEvent` 替代)
2. 聚合根**登记**事件(`events.add(...)`),usecase **禁止**手动 publish
3. Outbox 写入在 `adapter/persistence/` 的 Repository 实现内,与聚合保存**同事务**(transactional outbox 模式)
4. Spring `ApplicationEventPublisher` / `@EventListener` / Kafka 客户端等基础设施**只允许**出现在 `adapter/messaging/`

## 7. ddd-spec.md 模板修订

### 7.1 §"接口约定"模板替换
旧版示例使用 `@Service`、`@Transactional` 暗示 Spring 用例层。
新版模板按下面结构产出:

```markdown
## 接口约定(初稿,可被 Superpowers 细化)

### Command(usecase 层 record,纯 Java)
```java
package com.example.ddd.<aggregate>.usecase;
public record <CommandName>(
    <UbiquitousLanguageType> <field>,
    ...
) {}
```

### UseCase 实现(usecase 层 POJO,零 Spring)
```java
package com.example.ddd.<aggregate>.usecase;
import com.example.ddd.<aggregate>.domain.*;
// ↑ 严禁 import org.springframework.* / jakarta.* / org.slf4j.*

public class <CommandName>UseCase implements UseCase<<CommandName>, <ReturnType>> {
    private final <Aggregate>Repository repo;
    public <CommandName>UseCase(<Aggregate>Repository repo) { this.repo = repo; }

    @Override
    public <ReturnType> execute(<CommandName> cmd) {
        // 业务规则委托给聚合根,这里只编排
    }
}
```

### 装配(framework 层)
```java
package com.example.ddd.<aggregate>.framework.config;

@Configuration
class <Aggregate>UseCaseConfig {
    @Bean
    UseCase<<CommandName>, <ReturnType>> <commandName>UseCase(
            <Aggregate>Repository repo, ...) {
        return new TransactionalUseCaseDecorator<>(
            new <CommandName>UseCase(repo, ...));
    }
}
```
```

### 7.2 §"Guardrails for Superpowers"扩充
新增 §6.4 中的反模式硬清单(完整复制),并加一条:

> Superpowers 在 `executing-plans` 阶段每写完一个 usecase 类后,**必须**:
> 1. `grep -r "org.springframework\|jakarta\.\|org.slf4j" src/main/java/.../usecase/` 期望零命中
> 2. 触发 ArchUnit 测试(若已生成),期望全绿

## 8. ddd-model.md 模板微调

在 "## 反模式(必须拒绝)" 节末尾追加:

> ❌ **领域层污染**:`domain/**/*.java` 出现任何 framework / Lombok / SLF4J import。
> Domain 是同心圆最内层,**理论上一个 jar 就能编译**。如果 domain 类需要日志或 ID 生成,定义端口接口,实现在 adapter 层。

## 9. README-DDD-HARNESS.md 微调

在新项目入门章节增加:
1. 4 环结构示意图(本文档 §3.1 的 4 行)
2. 一份精简的 CreateOrder 范例片段(本文档 §5.2 + §5.4 截选)
3. 引用 `architecture/CleanArchitectureTest.java`,提示 "这是 harness 硬执法者,删除等于放弃 4 环约束"

## 10. ArchUnit 模板新增

新文件:`src/templates/root/CleanArchitectureTest.java`
落位:`<目标项目>/src/test/java/architecture/CleanArchitectureTest.java`

涵盖规则(对应 §6.4 反模式):

```java
@AnalyzeClasses(packages = "com.example.ddd", importOptions = DoNotIncludeTests.class)
class CleanArchitectureTest {

    @ArchTest static final ArchRule domainPureOfFrameworks = noClasses()
        .that().resideInAPackage("..domain..")
        .should().dependOnClassesThat().resideInAnyPackage(
            "org.springframework..", "jakarta.persistence..", "jakarta.inject..",
            "org.slf4j..", "lombok..");

    @ArchTest static final ArchRule usecasePureOfFrameworks = noClasses()
        .that().resideInAPackage("..usecase..")
        .should().dependOnClassesThat().resideInAnyPackage(
            "org.springframework..", "jakarta.persistence..", "jakarta.inject..",
            "org.slf4j..", "lombok..");

    @ArchTest static final ArchRule transactionalOnlyInDecorator = classes()
        .that().areAnnotatedWith(Transactional.class).or()
              .containAnyMethodsThat(annotatedWith(Transactional.class))
        .should().haveFullyQualifiedName(
            "com.example.ddd.shared.framework.transaction.TransactionalUseCaseDecorator");

    @ArchTest static final ArchRule layeredDependencies = layeredArchitecture()
        .consideringAllDependencies()
        .layer("domain").definedBy("..domain..")
        .layer("usecase").definedBy("..usecase..")
        .layer("adapter").definedBy("..adapter..")
        .layer("framework").definedBy("..framework..")
        .whereLayer("framework").mayNotBeAccessedByAnyLayer()
        .whereLayer("adapter").mayOnlyBeAccessedByLayers("framework")
        .whereLayer("usecase").mayOnlyBeAccessedByLayers("framework", "adapter")
        .whereLayer("domain").mayOnlyBeAccessedByLayers("framework", "adapter", "usecase");

    @ArchTest static final ArchRule webControllerNoDomain = noClasses()
        .that().resideInAPackage("..adapter.web..")
        .should().dependOnClassesThat().resideInAPackage("..domain..");

    @ArchTest static final ArchRule repositoryImplLocation = classes()
        .that().areAnnotatedWith(Repository.class)
        .should().resideInAPackage("..adapter.persistence..");

    @ArchTest static final ArchRule sagaHandlersOnlyInMessaging = classes()
        .that().areAnnotatedWith(Component.class).and()
              .containAnyMethodsThat(annotatedWith(EventListener.class))
        .should().resideInAPackage("..adapter.messaging..");

    @ArchTest static final ArchRule eventClassesInDomain = classes()
        .that().implement(DomainEvent.class)
        .should().resideInAPackage("..domain..events..");

    @ArchTest static final ArchRule eventListenersOnlyInMessaging = methods()
        .that().areAnnotatedWith(EventListener.class)
        .should().beDeclaredInClassesThat().resideInAPackage("..adapter.messaging..");
}
```

> **ArchUnit 表达不到的两条规则**(降级为 grep / 人工 review):
> - "Saga handler 不出现 if/else 业务分支" — ArchUnit 不擅长方法体语义检查,改由 ddd-spec Guardrails 在 review 时强调
> - "所有 usecase 必须经 TransactionalUseCaseDecorator" — 难用 ArchUnit 直接表达,改由 R8 第 3 条强制 + Config 自审 + 单元测试覆盖确保

README 中提示:本测试由模板提供初版,项目特定规则请在此文件追加,但不要删除已有规则。

## 11. ddd-run CLI 改造

### 11.1 新增模板文件
- `src/templates/root/CleanArchitectureTest.java`(本文档 §10 的内容)

### 11.2 `src/commands/init.rs` 扩展
- 新增写入路径:`<target>/src/test/java/architecture/CleanArchitectureTest.java`
- 用 `include_str!()` 嵌入,与现有模板写入逻辑同构
- 保持 `--minimal` / `--force` / `--dir` 行为兼容

### 11.3 `src/commands/status.rs` 扩展
- 校验 checklist 增加一项:`src/test/java/architecture/CleanArchitectureTest.java` 是否存在
- 新增分类标题"ArchUnit 守卫"(可选,与现 8 项资产并列)

### 11.4 `README.md`(ddd-run 自身)更新
- 在"What ddd-run installs"章节加上 ArchUnit 模板说明
- 在"Why"章节补充"修订 v2:解决用例层 Spring 污染"

## 12. 验收标准

- [ ] `cargo install --path .` 后,新目录运行 `ddd-run init` 产出含完整 4 环目录骨架的指引(在 README-DDD-HARNESS.md 中)
- [ ] `ddd-run status` 校验通过,新模板文件被识别
- [ ] 新版 `/ddd-spec <用例名>` 产出的 spec 文档接口约定段使用纯 POJO usecase + framework Config 范式
- [ ] 新版 `CLAUDE.md` 含 R7(改) / R8 / R9 节
- [ ] 提供的 `CleanArchitectureTest.java` 在编译目标项目后能跑起来,对反例(故意在 usecase 加 `@Service`)能阻塞
- [ ] `ddd-storm.md` / `DOMAIN.md` 模板**未变动**,保持向后兼容

## 13. 范围外

- `ddd-springboot-demo` 自身的合规化重构(单独 PR / 单独任务)
- 非 JVM 栈(Rust / Go / Node)的等价方案(将来扩展模板族)
- IDE 插件 / lint 规则(ArchUnit + grep 已足够)
- 引入 Lombok / MapStruct / 其他工具的指引(让目标项目自己 brainstorming 决定,但 domain/usecase 始终禁用)

## 14. 决策溯源(brainstorm 关键问题)

| 问题 | 选项 | 取值 | 理由摘要 |
|---|---|---|---|
| 架构风格 | A 同心圆 / B 六边形 / C 保留现命名 | **A** | 用户明确"严格 4 层向内依赖" |
| 装配方式 | a 纯 POJO+Config / b @Inject / c @Component 单注解 | **a** | 与 Bob "用例可离开 framework 跑"金标准最契合 |
| 事务 | T1 入站注解 / T2 装饰器 / T3 TxPort | **T2** | 事务边界 = 用例边界,多入口共享 |
| T2 风味 | α 编程式 / β 装饰器 + @Transactional | **β** | ArchUnit 可一行规则锁死位置 |
| SLF4J | 允许 / 禁止 | **禁止** | 维持"用例零外部依赖"金标准 |
| 查询事务 | 统一装饰 / 仅命令 | **统一** | 语义一致,避免遗漏 |
| ArchUnit | 引入 / 不引入 / 可选 | **默认引入** | 规则机械化,CI 阻塞 |
