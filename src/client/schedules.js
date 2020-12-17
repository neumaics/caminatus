import React from 'react';

export class Schedules extends React.Component {
  constructor(props) {
    super(props);
    this.state = { schedules: [] };
  }

  componentDidMount() {
    fetch('http://localhost:8080/schedules')
      .then(response => response.json())
      .then(data => { this.setState({ schedules: data })});
  }

  render() {
    const { schedules } = this.state;
    const components = schedules.map(schedule => <Schedule key={schedule} scheduleId={schedule} />)
    return (<div>
      {components}
    </div>);
  }
}

export class Schedule extends React.Component {
  constructor(props) {
    super(props);
    this.state = { id: props.scheduleId };
  }

  componentDidMount() {
    fetch(`http://localhost:8080/schedules/${this.state.id}`)
      .then(response => response.json())
      .then(data => this.setState({ ...this.state, ...data }));
  }

  render() {
    const { name } = this.state;

    return (
      <span>{name}</span>
    );
  }
}
