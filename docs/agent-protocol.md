# Agent 通信协议文档

本文档描述 Vanmoi Agent（被控端）与主控端之间的通信协议。

## 概述

Agent 通过 HTTP/WebSocket 与主控端通信，主要包含以下流程：

1. **注册** - Agent 首次连接时向主控端注册
2. **基本信息上报** - Agent 启动时上报服务器硬件信息
3. **实时数据上报** - Agent 定期上报监控数据（HTTP 或 WebSocket）

## 认证方式

Agent 使用 Token 认证。注册成功后获得唯一 Token，后续请求需在 HTTP Header 中携带：

```
Authorization: Bearer <token>
```

---

## API 端点

### 1. 注册 Agent

**请求**

```
POST /api/agent/register
Content-Type: application/json

{
  "name": "My Server"  // 可选，默认 "New Server"
}
```

**响应**

```json
{
  "uuid": "550e8400-e29b-41d4-a716-446655440000",
  "token": "vmoi_a1b2c3d4e5f6..."
}
```

**说明**
- `uuid`: Agent 唯一标识符
- `token`: 认证令牌，需妥善保存

---

### 2. 上报基本信息

Agent 启动时调用，上报服务器硬件信息。

**请求**

```
POST /api/agent/info
Authorization: Bearer <token>
Content-Type: application/json

{
  "cpu_name": "Intel(R) Xeon(R) E5-2680 v4",
  "arch": "x86_64",
  "cpu_cores": 8,
  "os": "Ubuntu 22.04.3 LTS",
  "kernel_version": "5.15.0-91-generic",
  "gpu_name": "",
  "virtualization": "kvm",
  "mem_total": 17179869184,
  "swap_total": 2147483648,
  "disk_total": 107374182400,
  "version": "0.1.0",
  "ipv4": "1.2.3.4",
  "ipv6": "2001:db8::1"
}
```

**字段说明**

| 字段           | 类型   | 说明                                |
| -------------- | ------ | ----------------------------------- |
| cpu_name       | string | CPU 型号                            |
| arch           | string | 架构 (x86_64, aarch64 等)           |
| cpu_cores      | int    | CPU 核心数                          |
| os             | string | 操作系统名称和版本                  |
| kernel_version | string | 内核版本                            |
| gpu_name       | string | GPU 型号（可选）                    |
| virtualization | string | 虚拟化类型 (kvm, vmware, docker 等) |
| mem_total      | int64  | 内存总量（字节）                    |
| swap_total     | int64  | 交换分区总量（字节）                |
| disk_total     | int64  | 磁盘总量（字节）                    |
| version        | string | Agent 版本                          |
| ipv4           | string | IPv4 地址（可选）                   |
| ipv6           | string | IPv6 地址（可选）                   |

**响应**

```json
{"status": "ok"}
```

---

### 3. 上报监控数据（HTTP）

**请求**

```
POST /api/agent/report
Authorization: Bearer <token>
Content-Type: application/json

{
  "cpu": 45.5,
  "gpu": 0,
  "ram": 8589934592,
  "ram_total": 17179869184,
  "swap": 0,
  "swap_total": 2147483648,
  "load": 1.25,
  "temp": 55.0,
  "disk": 42949672960,
  "disk_total": 107374182400,
  "net_in": 1048576,
  "net_out": 524288,
  "net_total_up": 107374182400,
  "net_total_down": 214748364800,
  "process": 150,
  "connections": 42,
  "connections_udp": 10,
  "uptime": 86400
}
```

**字段说明**

| 字段            | 类型  | 说明                     |
| --------------- | ----- | ------------------------ |
| cpu             | float | CPU 使用率 (0-100)       |
| gpu             | float | GPU 使用率 (0-100，可选) |
| ram             | int64 | 已用内存（字节）         |
| ram_total       | int64 | 内存总量（字节）         |
| swap            | int64 | 已用交换分区（字节）     |
| swap_total      | int64 | 交换分区总量（字节）     |
| load            | float | 系统负载（1分钟）        |
| temp            | float | CPU 温度（°C，可选）     |
| disk            | int64 | 已用磁盘（字节）         |
| disk_total      | int64 | 磁盘总量（字节）         |
| net_in          | int64 | 网络入站速率（字节/秒）  |
| net_out         | int64 | 网络出站速率（字节/秒）  |
| net_total_up    | int64 | 总上传流量（字节）       |
| net_total_down  | int64 | 总下载流量（字节）       |
| process         | int   | 进程数                   |
| connections     | int   | TCP 连接数               |
| connections_udp | int   | UDP 连接数               |
| uptime          | int64 | 系统运行时间（秒）       |

**响应**

```json
{"status": "ok"}
```

---

### 4. WebSocket 实时上报

对于需要低延迟的场景，Agent 可通过 WebSocket 持续上报数据。

**连接**

```
GET /api/agent/ws
Authorization: Bearer <token>
Connection: Upgrade
Upgrade: websocket
```

**消息格式**

连接建立后，Agent 定期发送 JSON 格式的监控数据（与 HTTP 上报格式相同）：

```json
{
  "cpu": 45.5,
  "ram": 8589934592,
  "ram_total": 17179869184,
  ...
}
```

**建议上报间隔**: 5 秒

**心跳**: WebSocket 自动 Ping/Pong

---

## 实现建议

### Rust Agent 示例

```rust
use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
struct Report {
    cpu: f32,
    ram: i64,
    ram_total: i64,
    // ... 其他字段
}

async fn report(client: &Client, base_url: &str, token: &str, report: &Report) {
    client
        .post(format!("{}/api/agent/report", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(report)
        .send()
        .await
        .ok();
}
```

### 数据采集建议

- **CPU**: 读取 `/proc/stat` 计算使用率
- **内存**: 读取 `/proc/meminfo`
- **磁盘**: 使用 `statvfs` 系统调用
- **网络**: 读取 `/proc/net/dev` 计算速率
- **负载**: 读取 `/proc/loadavg`
- **进程数**: 计算 `/proc` 下的进程目录

---

## 错误处理

| HTTP 状态码 | 说明             |
| ----------- | ---------------- |
| 200         | 成功             |
| 401         | Token 无效或过期 |
| 400         | 请求格式错误     |
| 500         | 服务器内部错误   |

当收到 401 错误时，Agent 应尝试重新注册。
