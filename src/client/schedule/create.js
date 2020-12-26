import React from 'react';
import styled from 'styled-components';
import { useForm, useFieldArray } from 'react-hook-form';

import { TEMPERATURE_SCALE, TIME_SCALE, STEP_TYPE, toServiceSchema, save } from './common';

function Schedule() {
  this.name = '';
  this.description = '';
  this.scale = TEMPERATURE_SCALE.CELSIUS;
  this.steps = [];

  return this;
}

function Step() {
  this.description = '';
  this.startTemperature = 0.0;
  this.endTemperature = 0.0;
  this.type = STEP_TYPE.DURATION;
  this.unit = TIME_SCALE.HOURS;
  this.stepValue = 0.0;

  return this;
}

const Form = styled.form`
  display: grid;
`;

export const CreateSchedule = () => {
  const initialState = new Schedule();
  initialState.steps.push(new Step());
  initialState.steps.push(new Step());

  const { register, control, handleSubmit, watch } = useForm({ defaultValues: initialState });
  const { fields, append, remove } = useFieldArray({ control, name: 'steps' });

  const saveSchedule = (data) => {
    save(toServiceSchema(data));
  };

  const addStep = () => append(new Step());

  const removeStep = (event, index) => {
    event.preventDefault();
    remove(index);
  };

  return (
    <Form onSubmit={handleSubmit(saveSchedule)}>
      <label>
        <span>Name:</span>
        <input
          type='text'
          name='name'
          ref={register({ required: true, maxLength: 40, minLength: 2 })}
          autoComplete='off'
        />
      </label>
      <label>
        <span>Description:</span>
        <input
          type='text'
          name='description'
          ref={register}
          autoComplete='off'
        />
      </label>
      <fieldset>
        {Object.values(TEMPERATURE_SCALE).map(s => {
          return (
            <label key={s}>
              {s}:
              <input name='scale' ref={register()} type='radio' value={s}></input>
            </label>);
        })}
      </fieldset>
      
      {fields.map((step, i) => {
        const watchType = watch('steps', STEP_TYPE.DURATION);
        return (
          <div key={step.id}>
            <button onClick={(e) => removeStep(e, i)}>-</button>
            <input
              name={`steps[${i}].startTemperature`}
              defaultValue={step.startTemperature}
              ref={register()}
              type='number'
              step='0.01'
              min='0'
            />
            <input
              name={`steps[${i}].endTemperature`}
              defaultValue={step.endTemperature}
              ref={register()}
              type='number'
              step='0.01'
              min='0'
            />
            <select name={`steps[${i}].type`} ref={register()} defaultValue={step.type}>
              <option value={STEP_TYPE.RATE}>by</option>
              <option value={STEP_TYPE.DURATION}>over</option>
            </select>
            <input 
              name={`steps[${i}].stepValue`}
              type='number'
              defaultValue={step.stepValue}
              ref={register()}
              min='0'
              step='1'
            />
            {watchType[i].type === STEP_TYPE.RATE ? <span>per</span> : <span></span>}
            <select name={`steps[${i}.unit]`} ref={register()} defaultValue={step.unit}>
              {Object.values(TIME_SCALE).map(t => <option key={t} value={t}>{t}</option>)}
            </select>
          </div>); }
      )}
      <button onClick={addStep}>+</button>
      <input type='submit'></input>
    </Form>
  );
};
