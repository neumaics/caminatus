import React from 'react';

export class Schedules extends React.Component {
  constructor(props) {
    super(props);
    this.state = { schedules: [] };
  }

  componentDidMount() {
    fetch('http://localhost:8080/schedules')
      .then(response => response.json())
      .then(data => { console.log(data); this.setState({ schedules: data })});
  }

  render() {
    const { schedules } = this.state;
    return <div>
      {schedules.map(schedule => 
        <a href={`http://localhost:8080/schedules/${schedule}`}>{schedule}</a>
      )}
    </div>;
  }
}
