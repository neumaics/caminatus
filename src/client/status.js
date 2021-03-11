import React, { useState, useContext, useEffect } from 'react';
import styled from 'styled-components';

import { ServerEventsContext } from './server-events';

const Readout = styled.a`
  font-family: 'IBM Plex Mono', sans-serif;
`;

const StatusBarDefault = () => {
  const [temp, setTemp] = useState(0.0.toFixed(2));
  const [setPoint, setSetPoint] = useState(0.0.toFixed(2));
  const [state, setState] = useState('Idle');

  const c = useContext(ServerEventsContext);

  useEffect(() => {
    c && typeof c.register === 'function' && c.register('kiln', (message) => {
      const data = JSON.parse(message.data);

      setState(data.state);
      setTemp(data.temperature.toFixed(2));
      setSetPoint(data.setPoint.toFixed(2));
    });
  }, [c]);

  return (
    <div>
      <a>temperature</a>
      <Readout>{temp}°C</Readout>
      <a>set point</a>
      <Readout>{setPoint}°C</Readout>
      <Readout>{state}</Readout>
    </div>
  );
};

export const StatusBar = styled(StatusBarDefault)`
  background-color: #2C3033;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  justify-content: center;
  list-style-type: none;
  width: 60px;
  height: 100%;

  padding: 0;
  margin: 0;

  grid-column: 2 / 2;
`;
