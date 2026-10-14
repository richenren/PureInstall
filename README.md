# PureInstall

A lightweight, open-source Windows app installer and manager. Clean, ad-free, and easy to use.

PureInstall 的目标很简单：**只在需要时打开，帮你安装可信的常用软件，并看清电脑里已经装了什么。**

它不是杀毒软件，也不是传统“电脑管家”。

## 当前功能（V0.1）

- 精选常用 Windows 软件目录
- 基于 winget 的单个/批量安装
- 扫描 PureInstall 安装前就已经存在的软件
- 显示版本、发行商、安装日期等基础信息
- 精选软件与本机已安装状态匹配
- 装机清单 JSON 导入/导出
- 不常驻后台
- 不创建开机启动项
- 不自动清理或删除软件

## 技术栈

- Tauri 2
- Vue 3 + TypeScript
- Rust
- Windows Registry
- Windows Package Manager (winget)

## 开发环境

需要 Windows 10/11、Node.js 20+、Rust stable、Microsoft C++ Build Tools 和 WebView2 Runtime。

安装依赖并启动：

    npm install
    npm run tauri dev

构建：

    npm run tauri build

## 软件目录

精选软件维护在 src/data/apps.json。

每条记录包含软件名称、分类、winget Package ID、官方主页和用于识别本机安装状态的 aliases。

## 安全边界

PureInstall V0.1 坚持以下原则：

- 软件扫描只读，不修改注册表。
- 不使用 Win32_Product 做全量软件扫描。
- 不关闭、绕过或干扰 Defender、火绒、360 等安全产品。
- 不执行来源不明的 PowerShell 脚本。
- 不自动卸载或强制删除任何软件。
- 软件安装必须由用户主动触发。

## 文档

完整需求与验收标准见 [docs/REQUIREMENTS.md](docs/REQUIREMENTS.md)。

## Roadmap

- 软件变化快照（无需后台常驻）
- AI 体检报告导出
- 正常卸载入口
- 更丰富的精选软件目录
