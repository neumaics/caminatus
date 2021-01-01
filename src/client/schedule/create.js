import React from 'react';
import styled from 'styled-components';
import { useForm, useFieldArray } from 'react-hook-form';

import { TEMPERATURE_SCALE, TIME_SCALE, STEP_TYPE, toServiceSchema, save } from './common';
import { FormButton } from '../components';

// Adapted from https://richjenks.com/filename-regex/
const VALID_NAME_PATTERN = /^(?!.{256,})(?!(aux|clock\$|con|nul|prn|com[1-9]|lpt[1-9])(?:$|\.))[^ /\\][ \.\w-$()+=;#@~,&amp;']*[^\. \\\/]$/; 

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

const FormContainer = styled.div`
  display: flex;
  background-color: #51595E;
  border-radius: 0.5em;
  padding: 0.5em 1em;
  width: 100%;

  > form {
    width: 100%;
  }
`;

const FormField = styled.label`
  display: block;
  color: #C4C4C4;

  > input[type='text' i] {
    width: 100%;
    padding: 0.25em 0;
    border-radius: 0.5em;
    border: 2px solid transparent
  }
`;

// https://markheath.net/post/customize-radio-button-css
const TemperatureScaleSelect = styled.fieldset`
  border: none;
  padding: inherit;
  
  > legend {
    color: #C4C4C4;
  }

  > label {
    display: inline-block;
    background-color: #ddd;
    padding: 0.25em 0.75em;
    font-size: 16px;
    border: 2px solid transparent;
    border-radius: 0.5em;
    margin: 0 0.25em 0.25em 0.25em;
  }

  > input[type='radio'] {
    opacity: 0;
    position: fixed;
    width: 0;

    &:checked + label {
      background-color:#bfb;
      border-color: #4c4;
    }

    &:focus + label {
      border: 2px dashed: 444;
    }
  }
`;

const StepContainer = styled.div`
  display: block;

  > div {
    display: flex;
    margin: 0.25em 0 0.25em 1em;

    > input[type='number'] {
      width: 12ch;
      border-radius: 0.5em;
      border: 2px solid transparent;
      margin: 0 0.25em;

    }

    > select {
      border-radius: 0.5em;
    }
  }
`;

const hasError = (errors, i, field, type) => errors.steps && errors.steps[i] && errors.steps[i][field].type === type;

/**
 * @todo add configurable max temperature.
 */
export const CreateSchedule = () => {
  const initialState = new Schedule();
  initialState.steps.push(new Step());
  initialState.steps.push(new Step());

  const {
    control,
    errors,
    handleSubmit,
    register,
    setValue, 
    watch,
  } = useForm({ defaultValues: initialState });
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
    <FormContainer>
      <form onSubmit={handleSubmit(saveSchedule)}>
        <FormField>
          <span>Name</span>
          <input
            type='text'
            name='name'
            ref={register({ required: true, maxLength: 40, minLength: 1, pattern: VALID_NAME_PATTERN })}
            autoComplete='off'
          />
          {errors.name && errors.name.type === 'required' ? <span>required</span> : <span></span>}
          {errors.name && errors.name.type === 'maxLength' ? <span>name exceeds maximum length (40)</span> : <span></span>}
          {errors.name && errors.name.type === 'minLength' ? <span>name is too short (must be more than 1 character)</span> : <span></span>}
          {errors.name && errors.name.type === 'pattern' ? <span>name contains invalid characters</span> : <span></span>}
        </FormField>
        <FormField>
          <span>Description</span>
          <input
            type='text'
            name='description'
            ref={register({ maxLength: 512 })}
            autoComplete='off'
          />
          {errors.description && errors.description.type === 'maxLength' ? <span>exceeds maximum length (512)</span> : <span></span>}
        </FormField>
        <TemperatureScaleSelect>
          <legend>Temperature Scale</legend>
          {Object.values(TEMPERATURE_SCALE).map(s => {
            return (
              <React.Fragment key={s}>
                <input name='scale' ref={register()} type='radio' value={s}></input>
                <label onClick={() => setValue('scale', s)}>{s}</label>
              </React.Fragment>);
          })}
        </TemperatureScaleSelect>
        
        <StepContainer>
          {fields.map((step, i) => {
            const watchType = watch('steps', STEP_TYPE.DURATION);
            return (
              <div key={step.id}>
                <FormButton type='button' inverted={true} context='error' onClick={(e) => removeStep(e, i)}>-</FormButton>
                <input
                  name={`steps[${i}].startTemperature`}
                  defaultValue={step.startTemperature}
                  ref={register({ required: true, min: 0, max: 1400 })}
                  type='number'
                  step='0.01'
                  min='0'
                />
                {/* {hasError(errors, i, 'startTemperature', 'min') && <span>must be greater than 0</span> }
                {hasError(errors, i, 'startTemperature', 'max') && <span>must be less than 1400</span> } */}
                <input
                  name={`steps[${i}].endTemperature`}
                  defaultValue={step.endTemperature}
                  ref={register({ required: true, min: 0, max: 1400})}
                  type='number'
                  step='0.01'
                  min='0'
                />
                {/* { errors.steps && errors.steps[i] && errors.steps[i].endTemperature.type === 'min' && <span>must be greater than 0</span> }
                { errors.steps && errors.steps[i] && errors.steps[i].endTemperature.type === 'max' && <span>must be less than 1400</span> } */}
                <select name={`steps[${i}].type`} ref={register()} defaultValue={step.type}>
                  <option value={STEP_TYPE.RATE}>by</option>
                  <option value={STEP_TYPE.DURATION}>over</option>
                </select>
                <input 
                  name={`steps[${i}].stepValue`}
                  type='number'
                  defaultValue={step.stepValue}
                  ref={register({ required: true, min: 0 })}
                  min='0'
                  step='1'
                />
                {/* {errors.steps && errors.steps[i] && errors.steps[i].endTemperature.type === 'min' && <span>must be greater than 0</span>} */}
                {watchType[i].type === STEP_TYPE.RATE ? <span>per</span> : <span></span>}
                <select name={`steps[${i}.unit]`} ref={register({ required: true })} defaultValue={step.unit}>
                  {Object.values(TIME_SCALE).map(t => <option key={t} value={t}>{t}</option>)}
                </select>
              </div>); }
          )}
        </StepContainer>
        <FormButton inverted={true} context='default' onClick={addStep}>+</FormButton>
        <div></div>
        <FormButton context='default' type='submit'>Save</FormButton>
        <FormButton context='error' type='button'>Cancel</FormButton>
      </form>
    </FormContainer>
  );
};
