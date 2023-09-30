%%raw(`import './App.css'`)

@react.component
let make = () => {
  <div className="App">
    {"Hello World"->React.string}
  </div>
}
