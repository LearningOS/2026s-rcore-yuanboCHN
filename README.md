# rCore-Tutorial-Code

## 领取春夏季作业仓库

1. 加入 [2026 春夏季训练营](https://opencamp.cn/os2edu/camp/2026spring)，并绑定自己的 GitHub 账号。
2. 点击[领取作业仓库](https://github.com/LearningOS/2026s-enroll/issues/new?template=rcore.yml)，提交申请并接受仓库邀请。
3. 在回复的作业仓库中，按照下方教程完成实验并 push，在 Actions 和训练营网站查看成绩。

已领取过本课程的学员继续使用原作业仓库。

## Code

- [Source Code of labs](https://github.com/LearningOS/rCore-Tutorial-Code)

## Documents

- Concise Manual: [rCore-Tutorial-Guide](https://LearningOS.github.io/rCore-Tutorial-Guide/)

- Detail Book [rCore-Tutorial-Book-v3](https://rcore-os.github.io/rCore-Tutorial-Book-v3/)

## OS API docs of rCore Tutorial Code

- [OS API docs of ch1](https://learningos.github.io/rCore-Tutorial-Code/ch1/os/index.html)
  AND [OS API docs of ch2](https://learningos.github.io/rCore-Tutorial-Code/ch2/os/index.html)
- [OS API docs of ch3](https://learningos.github.io/rCore-Tutorial-Code/ch3/os/index.html)
  AND [OS API docs of ch4](https://learningos.github.io/rCore-Tutorial-Code/ch4/os/index.html)
- [OS API docs of ch5](https://learningos.github.io/rCore-Tutorial-Code/ch5/os/index.html)
  AND [OS API docs of ch6](https://learningos.github.io/rCore-Tutorial-Code/ch6/os/index.html)
- [OS API docs of ch7](https://learningos.github.io/rCore-Tutorial-Code/ch7/os/index.html)
  AND [OS API docs of ch8](https://learningos.github.io/rCore-Tutorial-Code/ch8/os/index.html)
- [OS API docs of ch9](https://learningos.github.io/rCore-Tutorial-Code/ch9/os/index.html)

## Related Resources

- [Learning Resource](https://github.com/LearningOS/rust-based-os-comp2025/blob/main/relatedinfo.md)

## Setup

先按 [实验环境配置](https://learningos.github.io/rCore-Tutorial-Guide/0setup-devel-env.html) 准备 Linux、Rust 和 QEMU，再克隆自己的作业仓库。章节中的 `rust-toolchain.toml` 指定课程 Rust 版本。

```bash
$ git clone https://github.com/LearningOS/2026s-rcore-[YOUR_USER_NAME].git
$ cd 2026s-rcore-[YOUR_USER_NAME]
```

## Build & Run

```bash
# setup build&run environment first
$ git clone https://github.com/LearningOS/rCore-Tutorial-Test.git user
$ git checkout ch$ID
$ cd os
# run OS in ch$ID
$ make run
```

If you want to use Docker, return to the assignment repository root after selecting a chapter branch, then run:
```bash
# After clone the `rCore-Tutorial-Test` repository to your local machine, you can use the following command to build and run:
$ cd ..
$ make build_docker
$ make docker
```

If you experience network issues when accessing foreign resources such as GitHub in Docker, you can follow the following suggestions according to your stage:

- Docker pull:
  1. use proxy: https://docs.docker.com/reference/cli/docker/image/pull/#proxy-configuration

  2. use available domestic source (self-search)

- Docker build: use proxy https://docs.docker.com/engine/cli/proxy/#build-with-a-proxy-configuration

- Docker run: use proxy option, related operations are similar to `Docker build`, can refer to the relevant materials by yourself


本仓库提供 `ch1` 至 `ch8`；将 `$ID` 替换为章节数字。第 9 章 API 文档作为拓展阅读保留。

## Grading

在已配置环境的作业仓库根目录运行。首次下载检查器；已有 `ci-user/` 时直接复用，并保留自己的代码与 `reports/` 中的实验报告。后续章节应保留此前报告。

```bash
# setup build&run environment first
$ git clone https://github.com/LearningOS/rCore-Tutorial-Checker.git ci-user
$ git clone https://github.com/LearningOS/rCore-Tutorial-Test.git ci-user/user
$ git checkout ch$ID
# check&grade OS in ch$ID with more tests
$ make -C ci-user test CHAPTER=$ID
```

Notice: $ID is from [3,4,5,6,8]

## 提交与查看成绩

完成代码和报告后，使用领取仓库的 GitHub 账号推送到对应的 `ch3`、`ch4`、`ch5`、`ch6` 或 `ch8` 分支。在 Actions 查看测试与上传结果，再到 OpenCamp 查看成绩。`main`、`ch1`、`ch2`、`ch7` 不计分；每个计分章节通过后计 100 分，累计最高 500 分。
