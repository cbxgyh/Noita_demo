# Noita Demo - Falling Sand Game

基于 [Slow Rush Studios 博客系列](https://www.slowrush.dev/news/) 使用 Rust 和 Bevy 引擎开发的像素物理游戏。

## 项目概述

这个项目实现了博客系列中描述的 falling sand 游戏的核心功能，包括：

- **原子物理系统**: 沙子、水、酸、火等粒子的物理模拟
- **刚体物理**: 使用 Rapier 物理引擎的刚体对象
- **物理世界桥接**: 原子物理与刚体物理之间的交互
- **平台跳跃**: 包含 coyote time 和 jump buffering 的流畅跳跃系统
- **像素完美渲染**: 像素艺术风格的渲染系统

## 运行主游戏

```bash
cargo run
```

游戏控制：
- **WASD/箭头键**: 移动玩家
- **空格**: 跳跃
- **鼠标左键**: 使用刷子工具绘制原子
- **鼠标右键**: 施放魔法
- **数字键 1-5**: 切换刷子类型 (沙子、水、石头、酸、火)
- **Q/W/E/R**: 生成不同类型的原子用于测试

### 编辑器控制：
- **F1**: 切换关卡编辑器模式
- **鼠标滚轮**: 调整刷子大小
- **F2**: 保存关卡
- **F3**: 加载关卡
- **Ctrl+Z/Y**: 撤销/重做

### 关卡切换：
- **[ ]**: 切换到上一个关卡类型
- **]**: 切换到下一个关卡类型

### 音效控制：
- **M**: 切换音效开/关
- **+/-**: 调整音量

### 触摸屏控制：
- **T**: 切换触摸屏模式
- **虚拟摇杆**: 移动控制
- **虚拟按钮**: 跳跃、施法、切换原子类型

## 示例程序

项目包含多个独立的示例程序，每个对应博客系列中的一篇文章：

### 1. 基础沙子物理 (The Game So Far)
```bash
cargo run --example falling_sand_basic
```
展示基本的沙子和水粒子物理。

### 2. 平台跳跃系统 (Making Platforming Feel Good)
```bash
cargo run --example platforming_jump
```
展示带有 coyote time 和 jump buffering 的流畅平台跳跃。

### 3. 物理世界桥接 (Bridging Physics Worlds)
```bash
cargo run --example physics_bridge
```
展示原子物理与刚体物理之间的交互。

### 4. 优化物理桥接 (Optimizing the Physics Bridge)
```bash
cargo run --example optimizing_physics_bridge
```
展示耳剪辑三角剖分算法优化碰撞网格性能。

### 5. 原子动能 (Making Atoms Kinetic)
```bash
cargo run --example making_atoms_kinetic
```
展示原子如何获得速度和惯性，实现更真实的物理行为。

### 6. 粒子系统 (Particles, for real this time)
```bash
cargo run --example particles_real
```
展示粒子系统如何防止原子压碎移动物体。

### 7. 物体友好交互 (Playing Nice with Moving Bodies)
```bash
cargo run --example playing_nice_with_bodies
```
展示移动物体如何与原子（如水、沙子）正确交互，包括浮力和位移。

### 8. 魔法系统设计 (Designing a Magic System)
```bash
cargo run --example designing_magic_system
```
展示魔法施法系统、元素弱点和战斗机制。

### 9. 原子洞与多人游戏 (Atomic Holes and Multiplayer)
```bash
cargo run --example atomic_holes_multiplayer
```
展示带有洞的地形和基础多人游戏支持。

### 10. Discord与水花效果 (Discord and Making a Splash)
```bash
cargo run --example discord_making_splash
```
展示水物理、浮力系统和动态水花效果。

### 11. 酸与火 (Acid's Ire and Burning Fire)
```bash
cargo run --example acid_fire
```
展示腐蚀和燃烧的化学反应系统。

### 12. 沸腾与辛劳 (Boil and Toil)
```bash
cargo run --example boil_toil
```
展示蒸汽和毒药的生成，以及温度系统。

### 13. 多人游戏狂热梦想 (Multiplayer Fever Dreams)
```bash
cargo run --example multiplayer_fever
```
展示高级网络概念：回滚、预测和状态同步。

### 14. 休眠原子优化 (Let Sleeping Atoms Lie)
```bash
cargo run --example let_sleeping_atoms_lie
```
展示原子休眠系统以提高性能。

### 15. 关卡加载系统 (Loading Levels)
```bash
cargo run --example loading_levels
```
展示关卡序列化、保存和加载功能。

### 16. 射击系统 (Pew Pew Pew)
```bash
cargo run --example pew_pew_pew
```
展示武器、弹药和射击机制。

### 17. 布娃娃物理 (Ragdolls)
```bash
cargo run --example ragdolls
```
展示基于关节的角色物理动画。

### 18. 爆炸破坏 (Big Bada Boom)
```bash
cargo run --example big_bada_boom
```
展示物理破坏和爆炸效果。

### 19. 梯子和蘑菇 (Ladders and Shrooms)
```bash
cargo run --example ladders_and_shrooms
```
展示平台跳跃和交互世界对象。

### 20. 投掷粉末原子 (Flinging Powder Atoms)
```bash
cargo run --example flinging_powder_atoms
```
展示手榴弹和粉末物理。

## 项目结构

```
src/
├── main.rs          # 主程序入口
├── atoms.rs         # 原子物理系统
├── physics.rs       # 刚体物理桥接
├── rendering.rs     # 像素渲染系统
├── game.rs          # 游戏逻辑和玩家控制
examples/
├── falling_sand_basic.rs           # 基础沙子物理
├── platforming_jump.rs             # 平台跳跃系统
├── physics_bridge.rs                # 物理世界桥接
├── optimizing_physics_bridge.rs     # 耳剪辑优化
├── making_atoms_kinetic.rs          # 原子动能系统
├── particles_real.rs                # 粒子物理
├── playing_nice_with_bodies.rs      # 物体交互
├── designing_magic_system.rs        # 魔法系统设计
├── atomic_holes_multiplayer.rs      # 原子洞和多人游戏
├── discord_making_splash.rs         # 水花物理
├── acid_fire.rs                      # 酸和火的反应
├── boil_toil.rs                      # 蒸汽和毒药
├── multiplayer_fever.rs             # 高级网络
├── let_sleeping_atoms_lie.rs        # 原子休眠优化
├── loading_levels.rs                 # 关卡加载系统
├── pew_pew_pew.rs                    # 射击系统
├── ragdolls.rs                       # 布娃娃物理
├── big_bada_boom.rs                  # 爆炸破坏
├── ladders_and_shrooms.rs            # 梯子和蘑菇
├── flinging_powder_atoms.rs          # 粉末投掷
```

## 技术特性

### 原子物理系统
- 支持多种原子类型：空、沙子、水、酸、火、烟、蒸汽、毒药、石块
- 真实的物理属性：密度、流动性、气体行为
- 化学反应：火+水=蒸汽，酸+水=毒药等

### 刚体物理集成
- 使用 Bevy Rapier 2D 物理引擎
- 原子地形自动生成碰撞体
- 刚体对象可以推动和位移原子

### 流畅的平台跳跃
- **Coyote Time**: 离开地面后短暂时间内仍可跳跃
- **Jump Buffering**: 着陆前按跳跃键，着陆后自动跳跃
- 可变跳跃高度：松开跳跃键可控制跳跃高度

### 像素完美渲染
- 像素对齐的相机系统
- 像素艺术友好的渲染管线

## 博客系列对应

这个项目按照以下博客文章顺序实现了功能：

1. [The Game So Far](https://www.slowrush.dev/news/the-game-so-far) - 基础沙子模拟
2. [Choosing an Engine](https://www.slowrush.dev/news/choosing-an-engine) - 选择 Bevy 引擎
3. [Making Platforming Feel Good](https://www.slowrush.dev/news/making-platforming-feel-good) - 平台跳跃
4. [Pixel Perfect Rendering](https://www.slowrush.dev/news/pixel-perfect-rendering) - 像素渲染
5. [Bridging Physics Worlds](https://www.slowrush.dev/news/bridging-physics-worlds) - 物理桥接
6. [Optimizing the Physics Bridge](https://www.slowrush.dev/news/optimizing-the-physics-bridge) - 优化物理桥接

## 已完成功能 ✅

### 高级物理系统
- **原子动能和惯性系统**: 原子现在具有质量、速度、摩擦力和热传递
- **粒子系统优化**: 高效的粒子交互系统，防止原子重叠并模拟真实物理

### 魔法系统
- **魔法施法系统**: 基于 perks 的可编程魔法系统
- **法术组合**: 支持多种法术组合（Projectile、Explosion、Fire、Poison 等）
- **法术子程序**: 可修改法术行为的子程序系统

### 多人游戏支持
- **回滚网络**: 基于 "Rolling Back Sound" 博客的回滚网络系统
- **输入缓冲**: 延迟输入以补偿网络延迟
- **确定性锁步**: 确保客户端同步的确定性模拟

### 关卡系统
- **程序化关卡生成**: 使用噪声函数生成洞穴、岛屿、山脉、火山和实验室关卡
- **自定义关卡编辑器**: 像素级的关卡编辑器，支持绘制、填充、撤销/重做

### 音效系统
- **原子反应音效**: 火、酸、水等原子反应的动态音效
- **环境音效**: 基于原子浓度的环境声音
- **FMOD-like 架构**: 音频总线和效果系统

### 移动端支持
- **触摸屏控制**: 虚拟摇杆和按钮
- **手势识别**: 轻触、双击、滑动、长按等手势
- **移动端优化**: 针对移动设备的性能和UI优化

## 依赖

- `bevy` - 游戏引擎
- `bevy_rapier2d` - 2D 物理引擎
- `rand` - 随机数生成
- `noise` - 噪声生成（预留用于程序化生成）

## 开发计划

- [x] 基础原子物理系统
- [x] 刚体物理集成
- [x] 物理世界桥接
- [x] 平台跳跃控制
- [x] 像素完美渲染
- [ ] 原子动能和惯性
- [ ] 粒子系统
- [ ] 魔法系统
- [ ] 多人游戏支持
- [ ] 关卡编辑器
- [ ] 音效系统

## 许可证

## 项目规模

现在这个项目包含了 **21个完整的examples**，系统性地实现了 Slow Rush Studios 博客系列中的21篇核心文章，涵盖了从基础物理到高级游戏系统的完整技术栈。

本项目仅用于学习和演示目的。
