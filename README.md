# libotpl
The OTPL core library. provides parsing and compiling for the OTPL syntax.
OTPL 核心库，提供用于解析和编译OTPL语法。


### 项目结构

```
.
│  .gitignore
│  Cargo.toml
├─src
│  │  lib.rs                # 模块定义文件
│  └─core                   # otpl 库核心模块
│      │  mod.rs            # 模块定义文件
│      ├─ast                # 抽象语法树模块
│      │      mod.rs        # 模块定义文件
│      ├─opc                # 操作码定义模块
│      ├─parser             # 语法解析器
│      ├─scanner            # 词法分析器
│      └─token              # 词法定义模块
│
└─target                    # 编译临时文件
```




//Rustfmt failed at src/data_type.rs:424: line exceeded maximum length (maximum: 100, found: 149) (sorry)

