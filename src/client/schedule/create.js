import React from 'react';
import styled from 'styled-components';
import { useForm, useFieldArray } from 'react-hook-form';

import { TEMPERATURE_SCALE, toServiceSchema, save } from './common';
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

const validateStep = (stepText) => fetch(`http://${location.host}/step/parse/${encodeURIComponent(stepText)}`)
  .then(resp => resp.ok);

const FormContainer = styled.div`
  display: flex;
  background-color: #51595E;
  border-radius: 2px;
  padding: 0.5em 1em;
  width: 100%;
  overflow-y: scroll;

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
    border-radius: 2px;
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
    border-radius: 2px;
    margin: 0 0.25em 0.25em 0.25em;
  }

  > input[type='radio'] {
    opacity: 0;
    position: fixed;
    width: 0;

    &:checked + label {
      background-color:#2ECC71;
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
      border-radius: 2px;
      border: 2px solid transparent;
      margin: 0 0.25em;

    }

    > select {
      border-radius: 2px;
    }
  }
`;

export const CreateSchedule = () => {
  const initialState = new Schedule();
  initialState.steps.push({ text: '' });
  initialState.steps.push({ text: '' });

  const {
    control,
    errors,
    handleSubmit,
    register,
    setValue, 
  } = useForm({ defaultValues: initialState });
  const { fields, append, remove } = useFieldArray({ control, name: 'steps' });

  const saveSchedule = (data) => {
    save(toServiceSchema(data));
  };

  const addStep = () => append({ text: '' });

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
            ref={register({ maxLength: 128 })}
            autoComplete='off'
          />
          {errors.description && errors.description.type === 'maxLength' ? <span>exceeds maximum length (128)</span> : <span></span>}
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
            return (
              <div key={step.id}>
                <FormButton type='button' inverted={true} context='error' onClick={(e) => removeStep(e, i)}>-</FormButton>
                <input
                  type='text'
                  name={`steps[${i}].text`}
                  ref={register({ required: true, maxLength: 40, minLength: 1, validate: value => validateStep(value) })}
                  autoComplete='off'
                />
                {errors && errors.steps && errors.steps.length > 0 && errors.steps[i] && errors.steps[i].text ? <span>err</span> : <span></span>}
              </div>
            );
          })}
        </StepContainer>
        <FormButton inverted={true} context='default' onClick={addStep}>+</FormButton>
        <div></div>
        <FormButton context='default' type='submit'>Save</FormButton>
        <FormButton context='error' type='button'>Cancel</FormButton>
      </form>
    </FormContainer>
  );
};
