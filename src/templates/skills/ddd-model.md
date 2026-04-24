---
name: ddd-model
description: |
  触发条件:用户输入 /ddd-model,通常在 /ddd-storm 完成之后执行。
  基于事件风暴的输出,产出正式的 DDD 领域模型:识别 Entity / Value Object /
  Aggregate Root,划定聚合边界,建立 Ubiquitous Language 术语表,并**自动更新
  项目根目录下的 DOMAIN.md**(这是领域模型的 Single Source of Truth)。
  不写任何实现代码,只做建模。产出物后续会被 /ddd-spec 引用以生成 Superpowers spec。
  当用户说"画领域模型"、"划聚合边界"、"建 ubiquitous language"、"识别实体和值对象"
  时也应触发此技能。
---

# DDD Domain Modeling Skill

## 触发
用户使用 `/ddd-model` 命令。前置条件:`docs/ddd/` 下应存在至少一份 `01-event-storming-*.md`,
否则提示用户先运行 `/ddd-storm`。

## 目标
1. 基于事件风暴产出**正式领域模型**
2. 划定**聚合边界**
3. 建立**统一语言(Ubiquitous Language)术语表**
4. **更新项目根目录 `DOMAIN.md`**,作为所有后续 spec 与实现的引用锚点

## 工作流

### Step 1:读入事件风暴文档
读取 `docs/ddd/01-event-storming-*.md`(若有多份,询问用户选哪一份或合并)。

### Step 2:建模判定
对每个候选概念,按以下判定树分类:

```
概念 X
 ├─ 是"改状态 / 有不变式 / 需事务保护"的东西?
 │   ├─ 有独立身份标识 → Entity(若是聚合入口则 Aggregate Root)
 │   ├─ 由属性完全定义(Money、Address) → Value Object
 │   └─ 规则 / 动作
 │       ├─ 只依赖单个聚合 → 放在聚合根的方法里
 │       ├─ 跨聚合,无状态 → Domain Service
 │       └─ 涉及事务/消息/外部调用 → Application Service(不在 DOMAIN.md 讨论)
 │
 └─ 是"只读 / 查询 / 报表 / 列表"的东西(对应事件风暴 §6 读模型)?
     → 不是聚合,不进 §3 聚合清单。写入 DOMAIN.md §6 读模型章节。
       选定实现方式:
         - 单聚合直查:Repository.findById + DTO 映射
         - 跨聚合联合:Application Service 拼 DTO
         - 绕过聚合 SQL:JdbcTemplate 写宽表查询
         - 独立 Read Model 表:事件驱动重建
```

**重要**:事件风暴 `01-event-storming-*.md` §6 里的每个读模型都必须出现在 DOMAIN.md §6。
如果建模阶段丢了,查询侧到实现阶段就彻底消失。

### Step 3:聚合边界划定原则(强制遵守)

1. **一个事务只跨一个聚合**。如果业务规则要求两个聚合同时强一致,优先考虑合并;
   如果最终一致性可接受,用领域事件异步协调。
2. **外部只能通过聚合根访问聚合内对象**。`Order.getItems()` 返回不可变视图,
   不能让外部直接拿到 `OrderItem` 去修改。
3. **Repository 只对聚合根**。不要给 `OrderItem` 写 `OrderItemRepository`。
4. **聚合要小**。宁可拆成两个聚合 + 最终一致性,也不要一个巨大聚合锁死并发。
5. **聚合根 ID 引用**。跨聚合引用只用 ID,不持有对方对象。
   ✅ `Order { customerId: CustomerId }`
   ❌ `Order { customer: Customer }`

### Step 4:产出 `DOMAIN.md`(或更新)

**完整模板如下**(必须严格按此结构):

```markdown
# 领域模型 · <项目/限界上下文名>

> 本文档是本项目领域模型的 Single Source of Truth。
> 所有 Superpowers spec、代码命名、测试描述都必须使用本文档定义的 Ubiquitous Language。
> 禁止在代码中出现与本文档术语不一致的命名。

## 1. 限界上下文(Bounded Context)
- **名称**:<如 "积分管理上下文">
- **职责**:<一句话说清这个上下文负责什么>
- **不负责**:<明确列出与之相邻但不属于本上下文的职责>

## 2. 统一语言(Ubiquitous Language)

| 术语(中) | 术语(英) | 定义 | 类型 |
|---|---|---|---|
| 会员 | Member | 已注册并持有积分账户的用户 | Aggregate Root |
| 积分账户 | PointAccount | 记录会员当前可用积分与冻结积分 | Entity |
| 积分值 | Points | 不可为负的整数,以"分"为单位 | Value Object |
| 积分兑换 | PointRedemption | 会员发起的一次兑换行为 | Aggregate Root |
| ... | ... | ... | ... |

## 3. 聚合清单

### 3.1 <聚合名 A>
- **聚合根**:<AggregateRoot 名>
- **内部实体**:<Entity1>, <Entity2>
- **值对象**:<VO1>, <VO2>
- **不变式(Invariants)**:
  - INV-1: <必须始终成立的业务规则>
  - INV-2: ...
- **核心行为**:
  - `<方法签名>`:<做什么,前置条件,后置条件>
  - ...
- **发出的领域事件**:
  - `<EventName>`:<何时发出,携带什么数据>

### 3.2 <聚合名 B>
...(同上结构)

## 4. 跨聚合协调

列出跨聚合的协作关系,明确用"强一致"还是"最终一致":
- <聚合 A> 发出 `<Event>` → <聚合 B> 消费 → 触发 `<Command>`(最终一致)
- ...

## 5. 领域服务(Domain Services)
仅当规则跨多个聚合且无状态时才列出:
- `<ServiceName>.<method>`:<做什么,涉及哪些聚合>

## 6. 读模型(Read Models)

查询清单(对应 GET 端点)。**读模型不是聚合**,不受"一事务一聚合"约束。
必须把事件风暴 §6 的每个读模型都搬运到这里。

| 查询 | 输入 | 输出 DTO 字段 | 实现方式 | 备注 |
|---|---|---|---|---|
| ViewProductDetail | productId | productId, name, unitPrice, status | 单聚合直查 | 复用 ProductRepository.findById |
| BrowseProducts | page, size, keyword? | List<ProductSummary> + 分页 | 绕过聚合 SQL | 列表页用 |
| ViewMyCart | customerId | items[(product, qty, unitPrice, subtotal)], total | 跨聚合联合 | Cart + Product 拼装 |
| ... | | | | |

**实现方式枚举**:
- `单聚合直查` — `<Aggregate>Repository.findById` + 映射 DTO
- `跨聚合联合` — Application Service 拼 DTO
- `绕过聚合 SQL` — `JdbcTemplate` 直查
- `独立 Read Model 表` — 事件驱动重建(真正 CQRS)

## 7. 建模决策记录

每次重要的建模选择,简要记录一次,便于后续评审与迭代时说明:
- **决策 1**:为什么 `PointAccount` 是实体而不是值对象?
  → 因为同一会员的账户在不同时间点是"同一个东西"在演化,有身份。
- **决策 2**:为什么 `PointRedemption` 独立为聚合而不是 `Member` 的一部分?
  → 因为兑换记录有独立生命周期,且查询场景独立,放在 Member 内会导致聚合过大。

## 8. 下一步
- 对每个**命令用例**运行 `/ddd-spec <用例名>` 生成 Superpowers spec
- **实现 §6 的每个读模型**(简单查询直接实现,复杂查询可先用 `/ddd-spec` 生成查询 spec)
- spec 中所有命名必须引用本文档的 Ubiquitous Language
```

### Step 5:对话式建模
不要"一次性生成全部"。推荐交互节奏:

1. 先列出候选概念分类表,让用户确认
2. 逐个聚合细化,每个聚合成型后问用户"确认边界吗?"
3. 最后一起生成完整 `DOMAIN.md`

## 与其他 skill 的衔接

- **上游**:`docs/ddd/01-event-storming-*.md`
- **下游**:`/ddd-spec` 会读取 `DOMAIN.md` 中的术语
- **并列**:与 Superpowers 并列——`/ddd-model` 产出模型,Superpowers 产出代码

## 反模式(必须拒绝)

❌ **贫血模型**:聚合根只有 getter/setter,规则全在 Service 里
❌ **巨大聚合**:把订单、支付、物流塞进一个 `Order` 聚合
❌ **双向引用**:聚合之间互相持有对方对象
❌ **跨聚合事务**:一个事务同时修改两个聚合根
❌ **Repository 滥用**:给 VO 或内部 Entity 写 Repository

如用户坚持以上任一反模式,必须明确指出问题并给出 DDD 书中的依据,
然后由用户决定是否接受。

## 文件落位

- 建模过程记录:`docs/ddd/02-domain-model-<主题>.md`
- 最终产出:**更新项目根目录 `DOMAIN.md`**(这是给 `/ddd-spec` 读的)
