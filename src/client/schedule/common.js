
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

export const STEP_TYPE = {
  RATE: 'rate',
  DURATION: 'duration',
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

export const toServiceSchema = (clientSchedule) => {
  const service = {
    name: clientSchedule.name,
    description: clientSchedule.description || '',
    scale: clientSchedule.scale,
    steps: [],
  };

  clientSchedule.steps.reduce((acc, step) => {
    const s = {
      description: step.description || '',
      start_temperature: parseFloat(step.startTemperature),
      end_temperature: parseFloat(step.endTemperature),
      rate: null,
      duration: null,
    };

    if (step.type === STEP_TYPE.DURATION) {
      s.rate = { unit: step.unit, value: parseInt(step.stepValue, 10) };
    } else if (step.type === STEP_TYPE.RATE) {
      s.duration = { unit: step.unit, value: parseInt(step.stepValue, 10) };
    }
    
    acc.push(s);
    return acc;
  }, service.steps);

  return service;
};

export const save = (schedule) => 
  fetch(`http://${location.host}:8080/schedules`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
    },
    body: JSON.stringify(schedule)
  })
    .then(response => response.json());

export const byName = (name, normalized = false) => fetch(`http://${location.host}/schedules/${name}?normalize=${normalized}`)
  .then(response => response.json());


export const toGraphN = (normalized) => {
  const length = normalized.steps.length;

  return normalized.steps.reduce((acc, step, index) => {
    acc.push({
      x: step.start_time,
      y: step.start_temperature,
    });

    if (index === length - 1) {
      acc.push({
        x: step.end_time,
        y: step.end_temperature
      });
    }
    return acc;
  }, []);
};
