<template>
  <!-- thanks to https://codepen.io/virpo/pen/YzKWWPW -->
  <svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" viewBox="0 0 24 24" class="fill-current">
    <rect class="eq-bar eq-bar--1" x="4" y="4" width="3.7" height="8"/>
    <rect class="eq-bar eq-bar--2" x="10.2" y="4" width="3.7" height="16"/>
    <rect class="eq-bar eq-bar--3" x="16.3" y="4" width="3.7" height="11"/>
  </svg>
</template>

<style scoped>
.eq-bar {
  transform: scale(1, -1) translate(0, -24px);
}

.eq-bar--1 {
  animation-name: short-eq;
  animation-duration: 0.5s;
  animation-iteration-count: infinite;
  animation-delay: 0s;
}

.eq-bar--1 {
  animation-name: short-eq;
  animation-duration: 0.5s;
  animation-iteration-count: infinite;
  animation-delay: 0s;
}

.eq-bar--2 {
  animation-name: tall-eq;
  animation-duration: 0.5s;
  animation-iteration-count: infinite;
  animation-delay: 0.17s;
}

.eq-bar--3 {
  animation-name: short-eq;
  animation-duration: 0.5s;
  animation-iteration-count: infinite;
  animation-delay: 0.34s;
}

@keyframes short-eq {
  0% {
    height: 8px
  }

  50% {
    height: 4px
  }

  100% {
    height: 8px
  }
}

@keyframes tall-eq {
  0% {
    height: 16px
  }

  50% {
    height: 6px
  }

  100% {
    height: 16px
  }
}
</style>
