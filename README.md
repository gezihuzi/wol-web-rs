# WOL-Web (QuickConnect 公网安全版)

Rust 实现的 Wake-on-LAN 服务，支持网页操作、设备管理、配置持久化，并可通过群辉 QuickConnect 公网访问。

## 功能

- 内网广播 WOL 包，不暴露 UDP 到公网
- 强 Token 认证
- 设备列表管理（新增/删除/查看）
- 配置文件持久化 (`devices.json`)
- Docker 容器化部署
- 网页一键唤醒

## 构建镜像

```bash
docker build -t wol-web .
```

## 运行容器（持久化 + QuickConnect 公网访问）

```bash
docker run -d \
  --network=host \
  -v /volume1/docker/wol-web/config:/app/config \
  -e API_TOKEN=yourtoken \
  wol-web
```

- `--network=host`：保证 WOL 包可以发送到内网广播
- `/app/config` 挂载卷：持久化 `devices.json`
- `API_TOKEN`：公网访问时必须携带

## 使用网页前端

访问：

```
https://[QuickConnect ID]:8080/web/index.html
```

- 输入 Token 访问设备列表
- 点击设备按钮唤醒
- 新增/删除设备自动写入 `devices.json`

## API 文档

### 获取设备列表

```
GET /devices
```

返回 JSON 列表。

### 新增设备

```
POST /devices
Content-Type: application/json

{
  "name": "My-PC",
  "mac": "00:11:22:33:44:55",
  "token": "yourtoken"
}
```

### 删除设备

```
DELETE /devices?name=My-PC&token=yourtoken
```

### 唤醒设备

```
GET /wake?device_name=My-PC&token=yourtoken
```

可选参数 `ip`：广播地址，默认 `255.255.255.255`。

## 注意事项

1. Token 必须安全保管，防止公网滥用
2. Docker 必须使用 `--network=host`，否则 WOL 包无法发送
3. 配置文件挂载卷 `/volume1/docker/wol-web/config`，保证重启容器不丢失设备列表
4. QuickConnect 自动走 HTTPS，保证公网访问安全
