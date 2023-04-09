## AI ToolBox

A MVP solution for us to go through chatgpt api and get more information about openai capabilities


## TODOS
- 通过预读token数,可以确定可以拿到的上下文最大值,以确定能够搜索带上的上下文数量,也能确定每本书消耗的token数量 (能读到每本书消耗的量)
- 提高embedding的颗粒度,可以提高搜索的精确性 (直接用每一行作为embedding,只会提高请求数,但消耗token书是一致的)
- 通过压缩算法,提高可以传入的token数
- 将检索参数添加书本唯一标识符,防止多次请求embedding (完成)
- 限制多线程请求速率, 符合openai要求 (通过睡眠目前没问题)
- 使用本地的embedding算法,减少费用
- 向量数据库加上token数的metadata