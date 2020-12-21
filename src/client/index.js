import React from 'react';
import ReactDOM from 'react-dom';
import { createGlobalStyle } from 'styled-components';

import { App } from './app';
 
const GlobalStyle = createGlobalStyle`
html {
  height: 100%;
  background-color: #0A0F12;
  color: #fff;
  font-family: 'IBM Plex Sans', sans-serif;
}

body {
  height: 100%;
  padding: 0;
  margin: 0;
}

#app {
  height: 100%;
}
`;

const app = (
  <>
    <GlobalStyle />
    <App />
  </>
);

ReactDOM.render(app, document.getElementById('app'));
