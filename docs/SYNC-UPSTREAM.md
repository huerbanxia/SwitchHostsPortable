# 上游同步流程（Sync Upstream）

本仓库是 [oldj/SwitchHosts](https://github.com/oldj/SwitchHosts) 的便携（绿色）版 fork。
当**上游官方仓库更新**后，按本文档把新代码同步进本仓库，并把便携改动应用到新代码之上。

---

## 三处远程

| 远程 | 地址 | 角色 |
|---|---|---|
| `upstream` | GitHub `oldj/SwitchHosts` | 官方上游（更新的来源） |
| `origin` | GitHub fork `huerbanxia/SwitchHostsPortable` | 发布中心 |
| `ds` | 内网 Gitea `zgy/SwitchHostsPortable` | 内网备份 |

**约定**：`portable` 分支存放全部便携改动，`master` 分支与上游保持一致（仅偶尔同步一次）。

---

## 方式一：merge（推荐，省心）

合并保留两个来源的历史，之后每次上游更新不断 merge 即可。

```bash
# 1. 拉取上游最新（新提交 / 新 tag）
git fetch upstream

# 2. 切到 portable，把上游 master 合并进来
git checkout portable
git merge upstream/master

# 3. 有冲突则手动解决后：
#    git add <file> && git commit

# 4. 推送到 GitHub fork 和内网
git push origin portable
git push ds portable
```

- 若 merge 改写了工作区，便携改动与上游改动会同时保留在历史里。
- 若需要让绿色版包含上游新特性，合并后重新构建：
  ```bash
  npm install
  npm run tauri:build:portable    # 产出 dist/SwitchHostsPortable
  ```

---

## 方式二：rebase（历史更干净，需强推）

把便携提交重新铺到上游最新之上，历史近似线性补丁。

```bash
git fetch upstream
git checkout portable
git rebase upstream/master      # 把便携提交重放到上游最新
# 有冲突则依次解决，然后 git add <file> && git rebase --continue

git push --force origin portable
git push --force ds portable     # rebase 改写历史，需强推
```

⚠️ rebase 会改写提交哈希。若 fork 上已有其他人基于旧 `portable` 工作，强推会破坏他们的历史。**单人或自用再采用此方式。**

---

## master 分支同步（可选，保持原版基线对齐）

```bash
git fetch upstream
git checkout master
git merge upstream/master       # 或 git reset --hard upstream/master
git push origin master
git push ds master
```

---

## 冲突处理要点

便携改动集中在以下区域，上游若改动这些文件最可能冲突：

- `src-tauri/src/storage/portable.rs`（便携路径/检测）
- `src-tauri/src/storage/paths.rs`、`data_dir_pointer.rs`、`mod.rs`
- `src-tauri/src/lib.rs`（autostart / 更新 / 日志的便携分支）
- `src-tauri/src/migration/mod.rs`
- `script/build-portable.mjs`、`package.json`、`docs/portable.zh_cn.md`

冲突时：打开冲突文件，保留**便携逻辑**与**上游新功能**，删除 `<<<<<<< / ======= / >>>>>>>` 标记，`git add` 后继续。

---

## 先改写便携标记/版本（发布前）

便携版没改 Tauri `productName`（仍是 `SwitchHosts`），发布 zip 名不绑定版本号
（README 写"最新的 `SwitchHostsPortable.zip`"）。所以同步上游后**无需**改版本号相关文案。

---

## 例行同步命令（一条龙）

```bash
git fetch upstream && \
git checkout portable && \
git merge upstream/master && \
git push origin portable && \
git push ds portable && \
npm install && npm run tauri:build:portable
```