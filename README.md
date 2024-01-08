# wei-download

- [ ] 自动更新功能
    - [ ] 恢复所有种子
    - [ ] 种子移位到指定的目录
    - [x] 列出所有文件
    - [x] 删除下载，包含下载的文件
    - [x] 当下载一直为零，持续时间超过指定时间，自动删除，并退出自动更新程序
    - [x] 重新设置保存路径
        - [x] 确认文件是否100%下载完毕，如果没有下载完毕，等待下载完毕，然后再移动文件
    - [x] 如果是暂停状态，自动恢复下载
    - [x] 如果是异常状态，需要删除下载，并退出自动更新程序
    - [x] 正在下载的过程，删除文件，会导致100%完成，但是文件大小为零，这时候需要删除下载，并退出自动更新程序
        - [x] 停止下载，再恢复下载即可解决问题
        - [x] 中途修改文件，是否能正常使用，不能正常使用，需要删除下载，并退出自动更新程序
    - [x] 网络断开异常问题
    - [x] 文件大小检查
    - [x] 测试当更新文件已经下载到其它目录，这时候我们需要删除文件，然后重新下载