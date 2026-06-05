<template>
  <div class="about">
    <h1>关于 阿瓦隆</h1>
    <div class="content">
      <section>
        <h2>游戏简介</h2>
        <p>阿瓦隆（The Resistance: Avalon）是一款经典的身份推理类桌游，是《抵抗组织》的亚瑟王主题变体。游戏支持 5-10 人，分为正义方（蓝方）和邪恶方（红方）两个阵营。</p>
      </section>
      <section>
        <h2>阵营说明</h2>
        <ul>
          <li><strong>正义方</strong>（梅林、骑士）：好人，需要通过成功执行任务来获得胜利。梅林能识别邪恶方成员，但需要隐藏身份。</li>
          <li><strong>邪恶方</strong>（莫甘娜、刺客、爪牙）：坏人，通过破坏任务或刺杀梅林来获胜。莫甘娜伪装成梅林混淆视听。</li>
        </ul>
      </section>
      <section>
        <h2>游戏流程</h2>
        <ol>
          <li>每位玩家获得隐藏的身份角色</li>
          <li>随机选择队长，队长指派队员执行任务</li>
          <li>所有玩家投票——赞成或反对本次任务小队</li>
          <li>若通过，被指派的队员秘密投票——任务成功或失败</li>
          <li>邪恶方可以在任务中投失败票来破坏任务</li>
          <li>先获得指定数量任务成功的阵营获胜；若邪恶方破坏三次任务，则邪恶方获胜</li>
          <li>邪恶方若未破坏三次任务，可在最后阶段猜测梅林身份，猜中则邪恶方反败为胜</li>
        </ol>
      </section>
      <section>
        <h2>本项目</h2>
        <p>一个无服务器的局域网游戏辅助工具。在电脑或 Android 手机上启动服务器，朋友们通过浏览器扫码或输入 IP 即可加入游戏，无需安装任何额外软件。</p>
        <p class="tech">技术栈：Rust + Axum + Vue 3</p>
      </section>
    </div>
  </div>
</template>

<style scoped>
.about {
  max-width: 800px;
  margin: 0 auto;
  padding: 2rem;
}
h1 {
  color: #ffd700;
  text-align: center;
  margin-bottom: 2rem;
  font-size: 2rem;
}
.content {
  line-height: 1.8;
  font-size: 1.05rem;
}
section {
  margin-bottom: 2rem;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  padding: 1.5rem;
}
h2 {
  color: #ffd700;
  margin-bottom: 1rem;
  font-size: 1.3rem;
}
ul, ol {
  padding-left: 1.5rem;
}
li {
  margin-bottom: 0.5rem;
}
strong {
  color: #e0e0e0;
}
.tech {
  color: #888;
  font-size: 0.9rem;
  margin-top: 1rem;
  font-family: monospace;
}
</style>
