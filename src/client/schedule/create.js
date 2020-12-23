import React, { useState } from 'react';
import styled from 'styled-components';

import { TEMPERATURE_SCALE, TIME_SCALE } from './common';

function Schedule() {
  this.name = '';
  this.description = '';
  this.scale = '';
  this.steps = [];

  return this;
}

function Step() {
  this.description = '';
  this.startTemperature = 0.0;
  this.endTemperature = 0.0;
  this.duration = null;
  this.rate = null;

  return this;
}

function Rate() {
  this.value = 0;
  this.unit = '';
  return this;
}

function Duration() {
  this.value = 0;
  this.unit = '';
}

const Form = styled.form`
  display: grid;
`;

const InputGroup = styled.label`

`;

const initialState = new Schedule();
initialState.steps.push(new Step());
initialState.steps.push(new Step());

export const CreateSchedule = () => {
  const [schedule, setSchedule] = useState(initialState);

  const saveSchedule = (event) => {
    event.preventDefault();
  };

  const addStep = (event) => {
    event.preventDefault();
    schedule.steps.push(new Step());
    setSchedule({ ...schedule });
  };

  const removeStep = (event, index) => {
    event.preventDefault();
    schedule.steps.splice(index, 1);
    setSchedule({ ...schedule });
  };

  return (
    <Form onSubmit={saveSchedule}>
      <label>
        <span>Name:</span>
        <input
          type='text'
          value={schedule.name}
          onChange={(e) => setSchedule({ ...schedule, name: e.target.value })}
          autoComplete='off'
        />
      </label>
      <input placeholder='description' />
      <div>
        {Object.values(TEMPERATURE_SCALE).map(s => {
          return (
            <label key={s}>
              {s}:
              <input type='radio' name={s} value={s}></input>
            </label>);
        })}
      </div>
      
      {(schedule.steps.map((step, i) => (
        <div key={i}>
          <button onClick={(e) => removeStep(e, i)}>-</button>
          <select>
            <option>rate</option>
            <option>duration</option>
          </select>
          <input type='number' placeholder='start' value={step.startTemperature} />
          <input type='number' placeholder='end' value={step.endTemperature}/>
          <input type='number' placeholder='0.0' />
          <select>
            {Object.values(TIME_SCALE).map(t => <option key={t} value={t}>{t}</option>)}
          </select>
        </div>
      )))}
      <button onClick={addStep}>+</button>
      <input type='submit'></input>
    </Form>
  );
};