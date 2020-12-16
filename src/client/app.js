import React from 'react';

import { Schedules } from './schedules';

const App = ({ title }) => {
    
  return <div>
    <div>{title}</div>
    <Schedules />
  </div>;
}
 
export default App;
