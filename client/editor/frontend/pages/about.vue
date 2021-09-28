<template>
  <v-container fluid fill-height>
    <v-row justify="center">
      <Logo id="logo" />
      <blockquote class="blockquote">
        &#8220;There is no problem complex enough that Javascript cannot make
        worse.&#8221;
        <footer>
          <small>
            <em>&mdash;Julien Kauffmann</em>
          </small>
        </footer>
      </blockquote>
    </v-row>
  </v-container>
</template>

<style scoped>
.container {
  background: rgb(255, 255, 255);
  background: linear-gradient(
    180deg,
    rgba(255, 255, 255, 1) 0%,
    rgba(144, 144, 144, 1) 100%
  );
}

#logo {
  padding: 0 25%;
  height: auto;
  margin-bottom: 2em;
}

blockquote {
  text-align: center;
}
</style>