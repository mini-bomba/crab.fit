interface Person {
  name: string
  availability: string[]
}

interface Availability {
  date: string
  /** Names of everyone who is available at this date */
  people: string[]
}

interface AvailabilityInfo {
  availabilities: Availability[]
  /** The amount of people available in the date with lowest availability */
  min: number
  /** The amount of people available in the date with highest availability */
  max: number
}

/**
 * Takes an array of dates and an array of people,
 * where each person has a name and availability array, and returns the
 * group availability for each date passed in.
 */
export const calculateAvailability = (dates: string[], people: Person[]): AvailabilityInfo => {
  const availabilities: Availability[] = dates.map(date => {
    const names = people.flatMap(p => p.availability.some(d => d === date) ? [p.name] : [])
    return { date, people: names }
  })

  const min = availabilities.length === 0 ? 0 : Math.min(...availabilities.map((x) => x.people.length));
  const max = availabilities.length === 0 ? 0 : Math.max(...availabilities.map((x) => x.people.length));

  return { availabilities, min, max }
}
