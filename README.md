阅读《Writing An Interpreter In Go》，尝试用Rust实现一遍。

变更
* 在原作基础上加入浮点数的支持

问题
加入浮点数之后，由于浮点数是不能实现Eq的，从而导致包装这些数据的enum也无法实现Eq，到了书本最后一步实现Hash的时候
就发现没有办法直接使用HashMap来做了。

后记
作为一种解释器，这个基本流程我已经了解了，我本意是想从这里入手，看看能不能用Rust做一个脚本语言来作为我的MudServer
的一部分，目前来看这样并不是很合适，而一个更大的想法进入我的脑海。以前的MudOs就是用C实现了一套LPC的系统，包含了
LPC编程语言，同理可以用Rust来实现一个系统，也加入自己的编程语言。但这个课题似乎有那么点大，内心有点惧怕又出现一个
烂尾项目。最近查阅的资料，有两大方向
1. Rune 用rust实现的类似Rust的语言，这个应该是有很多可以借鉴的地方
2. WASCC 它有一套action的架构，看起来很有相似之处，可以研究是否可以在其基础上构建新的MudOs