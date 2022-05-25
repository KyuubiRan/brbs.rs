# brbs.rs
哔哩漫游黑名单高性能服务器

## 构建 & 运行
`cargo build --release & ./target/release/brbs-rs`

## 请求
### 查询
`请求`
```http 
GET /query/status/uid=123456
```
```http
GET /query/status/key=abcdefghijklmnopqrstuvwxyz
```
`响应`
```json
{"code": 200, "data": {"status": 1, "reason": "评论区发送解析链接"}}
```
| status | 状态 |
| :----: | :-: |
|   0    | 无 |
|   1    | 黑 |
|   2    | 白 |

### 查询被拉黑次数
`请求`
```http
GET /query/times/uid=123456
```
```http
GET /query/times/key=abcdefghijklmnopqrstuvwxyz
```
`响应`
```json
{"code": 200, "msg": "查询成功", "data": {"blackTimes": 3}}
```

### 修改状态
`请求`
```http
POST /admin/black

{"uid": 123456, "key": "...", "reason": "..."}
```
```http
POST /admin/white

{"uid": 123456, "key": "...", "reason": "..."}
```
```http
POST /admin/none

{"uid": 123456, "key": "...", "reason": "..."}
```
`响应`
```json
{"code": 200, "msg": "操作成功"}
```

### 最近一条记录
`请求`
```http
POST /admin/last

{"uid": 123456, "key": "..."}
```
`响应`
```json
{"code": 200, "msg":"查询成功", "data" {"status": 1, "reason": "评论区发送解析链接", "opRole": "admin", "timestamp": 1653490177054}}
```

### 添加/移除Admin Key
`请求`
```http
POST /owner/keygen

{"lvl": 1, "key": "...", "role": "..."}
```
**注意：** 其中lvl为可选参数，不填写默认为1 区间为\[0-127\]  
  
`响应`
```json
{"code": 200, "msg":"生成成功", "data":{"key":"..."}}
```
  
`请求`
```http
POST /owner/keyrevoke

{"key": "...", "role": "...", "revokeKey": "..."}
```
**注意：** `role`和`revokeKey`二选一   
  
`响应`
```json
{"code": 200, "msg": "操作成功"}
```
