import React from 'react';
import ReactDOM from 'react-dom';
 
import App from './app';

const title = 'Caminatus';
 
ReactDOM.render(
  <App title={title} />,
  document.getElementById('app')
);
