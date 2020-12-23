import React from 'react';
import styled from 'styled-components';

import { TEMPERATURE_SCALE, TIME_SCALE } from './common';

const Form = styled.form`
`;

export const CreateSchedule = () => {
  return (
    <Form>
      <input placeholder='name' />
      <input placeholder='description' />
      <select>
        {Object.values(TEMPERATURE_SCALE).map(s => <option key={s} value={s}>{s}</option>)}
      </select>
      <div>
        <button>-</button>
        <select>
          <option>rate</option>
          <option>duration</option>
        </select>
        <input type='number' placeholder='start' />
        <input type='number' placeholder='end' />
        <input placeholder='0.0' />
        <select>
          {Object.values(TIME_SCALE).map(t => <option key={t} value={t}>{t}</option>)}
        </select>
      </div>
      <button>+</button>
    </Form>
  );
};