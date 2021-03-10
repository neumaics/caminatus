
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

export const toServiceSchema = (clientSchedule) => {
  const service = {
    name: clientSchedule.name,
    description: clientSchedule.description || '',
    scale: clientSchedule.scale,
    steps: [],
  };

  service.steps = clientSchedule.steps.reduce((acc, step) => {
    acc.push(step.text);
    return acc;
  }, []);

  return service;
};

export const save = (schedule) => 
  fetch(`http://${location.host}/schedules`, {
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
