
export const TEMPERATURE_SCALE = {
  CELSIUS: 'Celsius',
  FAHRENHEIT: 'Fahrenheit',
  KELVIN: 'Kelvin',
};

export const TIME_SCALE = {
  HOURS: 'hours',
  HOUR: 'hour',
  MINUTES: 'minutes',
  MINUTE: 'minute',
  SECONDS: 'seconds',
  SECOND: 'second',
};

export const TIME_SECONDS = {
  [TIME_SCALE.HOURS]: 3600,
  [TIME_SCALE.HOUR]: 3600,
  [TIME_SCALE.MINUTES]: 60,
  [TIME_SCALE.MINUTE]: 60,
  [TIME_SCALE.SECONDS]: 1,
  [TIME_SCALE.SECOND]: 1,
};

export const durationToSeconds = (duration) => duration.value * TIME_SECONDS[duration.unit.toLowerCase()];
export const rateToSeconds = (step) => {
  const delta = step.end_temperature - step.start_temperature;
  const p = Math.abs(delta) / step.rate.value;
  const time = p * TIME_SECONDS[step.rate.unit.toLowerCase()];

  return time;
};

/**
 * Maps a schedule to a x/y axis datapoint form.
 * 
 * @fixme this is super inefficient.
 * @param {*} schedule 
 */
const toGraph = (schedule) => {
  const { norm } = schedule.steps.reduce((acc, step) =>{
    const endTime = step.duration ? durationToSeconds(step.duration) : rateToSeconds(step);

    if (acc.norm.find((s) => s.time === acc.time) === undefined) {
      acc.norm.push({ time: acc.time, temp: step.start_temperature});
    }
    
    if (acc.norm.find((s) => s.time === endTime) === undefined) {
      acc.norm.push({ time: acc.time + endTime, temp: step.end_temperature });
    }

    return {
      ...acc,
      time: acc.time + endTime
    };
  }, { norm: [], time: 0 });

  return norm;
};
