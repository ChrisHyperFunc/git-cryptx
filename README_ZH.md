# git-cryptx

[English](README.md) | 简体中文

自动加密/解密 Git 仓库中的敏感文件。
https://git-cryptx.201945.xyz

## 功能特点

- 🔒 透明的文件加密/解密
- 🔄 无缝集成 Git 工作流
- 🎯 精确的文件匹配控制
- 👥 支持团队协作
- 💻 跨平台支持

## 安装
bash
cargo install git-cryptx


## 快速开始
1. 初始化仓库：
git-cryptx init

2. 设置加密密钥：
git-cryptx set-key <your-key>

3. 配置需要加密的文件（编辑 .gitattributes）：
```
.secret filter=git-cryptx diff=git-cryptx
config/.key filter=git-cryptx diff=git-cryptx
sensitive/ filter=git-cryptx diff=git-cryptx
```


## 命令说明

- `init`: 初始化 git-cryptx
- `set-key <key>`: 添加加密密钥
- `rm-key`: 移除加密密钥
- `status`: 显示加密状态

## 工作原理

git-cryptx 使用 Git 的过滤器机制实现文件的自动加密和解密：

1. 当文件被添加到 Git 时，clean 过滤器自动加密内容
2. 当文件被检出时，smudge 过滤器自动解密内容
3. 工作目录中始终保持文件明文
4. Git 仓库中始终保持文件密文

## 安全说明

- 使用 AES-256-GCM 进行加密
- 密钥存储在 .git/cryptx 目录中
- 支持文件完整性验证
- 加密文件使用魔数标记

## 常见问题

Q: 如何与团队成员共享密钥？
A: 请通过安全渠道共享 .git/cryptx/keys/global_ase_key 文件。

Q: 如何查看加密文件的差异？
A: git-cryptx 支持直接查看加密文件的明文差异，使用普通的 git diff 即可。

Q: git pull 时提示本地文件会被覆盖怎么办？
A: 可以使用以下方法解决：

1. 如果确定本地文件没有修改，只是解密状态不同：
```bash
git-cryptx reset <file_path>
```

2. 如果本地文件确实有修改：
```bash
# 存储本地修改
git stash
# 拉取更新
git pull
# 恢复本地修改
git stash pop
```

3. 如果发生冲突，需要手动解决冲突后再提交。

## 团队协作

当新团队成员加入项目时，需要执行以下步骤：

1. 克隆仓库：
```bash
git clone <repository-url>
```

2. 初始化 git-cryptx：
```bash
git-cryptx init
```

3. 从团队其他成员处获取密钥文件：
   - 获取 `.git/cryptx/keys/global_ase_key` 文件
   - 将密钥文件放入本地仓库的相同位置
   - 或执行 git-cryptx set-key 密钥

4. 重新检出文件：
```bash
# Clean working directory
git clean -fd
# Checkout files to trigger decryption
git checkout .
```

5. 验证文件状态：
```bash
git-cryptx status
cat your-encrypted-file  # 检查文件是否正确解密
```

注意事项：
- 确保密钥文件正确放置
- 如果文件仍然是加密状态，尝试删除文件后重新检出
- 检查 git-cryptx status 确保所有配置正确

## 贡献指南

欢迎提交 Pull Request 或创建 Issue。

## 许可证

MIT License